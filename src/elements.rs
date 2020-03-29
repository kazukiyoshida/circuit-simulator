use super::simulator::Equation;
use nalgebra::base::{DMatrix, DVector};
use wasm_bindgen::prelude::*;

pub trait Element: std::fmt::Debug {
    // その要素が電流/電圧源であるか
    fn is_voltage_or_current_src(&self) -> bool {
        false
    }
    fn connect_pin_to_node(&mut self, pin_id: usize, node_id: usize);
    fn stamp(&self, eq: &mut Equation);
}

#[derive(Debug)]
pub struct Registor {
    id: usize,
    // 結合しているノードの id. デフォルトでは GND に結合している.
    //_この id が行列のインデックスに対応し、スタンプを押す.
    pins: [usize; 2],
    resistance: f32,
}

impl Registor {
    pub fn new(id: usize, registance: f32) -> Registor {
        let r = if registance == 0.0 { 0.01 } else { registance };
        Registor {
            id: id,
            pins: [0, 0],
            resistance: r,
        }
    }

    fn conductance(&self) -> f32 {
        1.0 / self.resistance
    }
}

impl Element for Registor {
    fn connect_pin_to_node(&mut self, pin_id: usize, node_id: usize) {
        self.pins[pin_id] = node_id;
    }

    fn stamp(&self, eq: &mut Equation) {
        match self.pins {
            [p0, 0] => {
                let p0 = *eq.node_index.get(&p0).unwrap();
                eq.a[(p0, p0)] += self.conductance();
            }
            [0, p1] => {
                let p1 = *eq.node_index.get(&p1).unwrap();
                eq.a[(p1, p1)] += self.conductance();
            }
            [p0, p1] => {
                let p0 = *eq.node_index.get(&p0).unwrap();
                let p1 = *eq.node_index.get(&p1).unwrap();
                eq.a[(p0, p0)] += self.conductance();
                eq.a[(p1, p1)] += self.conductance();
                eq.a[(p0, p1)] -= self.conductance();
                eq.a[(p1, p0)] -= self.conductance();
            }
        }
    }
}

// ダイオード
// 順方向電圧 Vd における電流 I(Vd) を区分線形近似でモデリングする.
// I(V) = 0                      ( Vd <= threshold )
//      = grad * (V - threshold) ( Vd  > threshold )
#[derive(Debug)]
pub struct Diode {
    id: usize,
    // pins[0]: Anode,  pins[1]: Cathode
    pins: [usize; 2],
    threshold: f32,
    grad: f32,
}

impl Diode {
    pub fn new(id: usize) -> Diode {
        Diode {
            id: id,
            pins: [0, 0],
            threshold: 0.674,
            grad: 0.191,
        }
    }

    pub fn current(&self, volt: f32) -> f32 {
        if volt <= self.threshold {
            0.0
        } else {
            self.grad * (volt - self.threshold)
        }
    }

    pub fn d_current(&self, volt: f32) -> f32 {
        if volt <= self.threshold {
            0.0
        } else {
            self.grad
        }
    }
}

impl Element for Diode {
    fn connect_pin_to_node(&mut self, pin_id: usize, node_id: usize) {
        self.pins[pin_id] = node_id;
    }

    // cf. https://spicesharp.github.io/SpiceSharp/articles/custom_components/modified_nodal_analysis.html
    fn stamp(&self, eq: &mut Equation) {
        let g = self.threshold * self.grad;
        match self.pins {
            [p0, 0] => {
                let p0 = *eq.node_index.get(&p0).unwrap();
                eq.z[p0] += g;

                let didv = self.d_current(eq.x[p0]);
                eq.a[(p0, p0)] += didv;
            }
            [0, p1] => {
                let p1 = *eq.node_index.get(&p1).unwrap();
                eq.z[p1] -= g;

                let didv = self.d_current(eq.x[p1]);
                eq.a[(p1, p1)] += didv;
            }
            [p0, p1] => {
                let p0 = *eq.node_index.get(&p0).unwrap();
                let p1 = *eq.node_index.get(&p1).unwrap();
                eq.z[p0] += g;
                eq.z[p1] -= g;

                let didv = self.d_current(eq.x[p0] - eq.x[p1]);
                eq.a[(p0, p1)] -= didv;
                eq.a[(p1, p0)] -= didv;
                eq.a[(p0, p0)] += didv;
                eq.a[(p1, p1)] += didv;
            }
        }
    }
}

#[derive(Debug)]
pub struct IndVoltageSrc {
    id: usize,
    pins: [usize; 2],
    voltage: f32,
}

impl IndVoltageSrc {
    pub fn new(id: usize, volt: f32) -> IndVoltageSrc {
        IndVoltageSrc {
            id: id,
            pins: [0, 0],
            voltage: volt,
        }
    }
}

impl Element for IndVoltageSrc {
    fn connect_pin_to_node(&mut self, pin_id: usize, node_id: usize) {
        self.pins[pin_id] = node_id;
    }

    fn is_voltage_or_current_src(&self) -> bool {
        true
    }

    fn stamp(&self, eq: &mut Equation) {
        let index = eq.src_index.get(&self.id).unwrap() + eq.node_index.len() - 1;
        eq.z[index] = self.voltage;

        match self.pins {
            [p0, 0] => {
                let p0 = *eq.node_index.get(&p0).unwrap();
                eq.a[(p0, index)] = 1.0;
                eq.a[(index, p0)] = 1.0;
            }
            [0, p1] => {
                let p1 = *eq.node_index.get(&p1).unwrap();
                eq.a[(p1, index)] = -1.0;
                eq.a[(index, p1)] = -1.0;
            }
            [p0, p1] => {
                let p0 = *eq.node_index.get(&p0).unwrap();
                let p1 = *eq.node_index.get(&p1).unwrap();
                eq.a[(p0, index)] = 1.0;
                eq.a[(p1, index)] = -1.0;
                eq.a[(index, p0)] = 1.0;
                eq.a[(index, p1)] = -1.0;
            }
        }
    }
}
