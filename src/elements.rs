use wasm_bindgen::prelude::*;
use nalgebra::base::{DMatrix, DVector};

// 各要素の参照関係
// Element 1 : N Pin M : 1 Node

// Node への参照
type NodeId = usize;

pub trait Element: std::fmt::Debug {
    // Element の pin を Node＿に繋げる
    fn connect_pin_to_node(&mut self, pin_id: usize, node_id: usize);

    // その要素が電圧源であるか
    fn is_voltage_src(&self) -> bool {
        false
    }
    // その要素が電流源であるか
    fn is_current_src(&self) -> bool {
        false
    }
    // その要素が電流/電圧源であるか
    fn is_voltage_or_current_src(&self) -> bool {
        self.is_voltage_src() || self.is_current_src()
    }

    // 修正節点法における回路の方程式の行列へのスタンプ
    fn stamp_matrix(&self, _: &mut DMatrix<f32>, _: &DVector<f32>) {}
    fn stamp_matrix_by_src(&self, _: &mut DMatrix<f32>, _: usize) {}

    // 修正節点法における回路の方程式のベクトルへのスタンプ
    fn stamp_vector(&self, _: &mut DVector<f32>, _: &DVector<f32>) {}
    fn stamp_vector_by_src(&self, _: &mut DVector<f32>, _: usize) {}
}

#[derive(Debug)]
pub struct Registor {
    pins: [NodeId; 2],
    resistance: f32,
}

impl Registor {
    pub fn new(registance: f32) -> Registor {
        let r = if registance == 0.0 { 0.01 } else { registance };
        Registor {
            pins: [0, 0],
            resistance: r,
        }
    }

    pub fn conductance(&self) -> f32 {
        1.0 / self.resistance
    }
}

impl Element for Registor {
    fn connect_pin_to_node(&mut self, pin_id: usize, node_id: usize) {
        self.pins[pin_id] = node_id;
    }

    fn stamp_matrix(&self, matrix: &mut DMatrix<f32>, _: &DVector<f32>) {
        match self.pins {
            [p0, 0] => {
                let p0 = p0 - 1;
                matrix[(p0, p0)] += self.conductance();
            }
            [0, p1] => {
                let p1 = p1 - 1;
                matrix[(p1, p1)] += self.conductance();
            }
            [p0, p1] => {
                let p0 = p0 - 1;
                let p1 = p1 - 1;
                matrix[(p0, p0)] += self.conductance();
                matrix[(p1, p1)] += self.conductance();
                matrix[(p0, p1)] -= self.conductance();
                matrix[(p1, p0)] -= self.conductance();
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
    // pins[0] : Anode
    // pins[1] : Cathode
    pins: [NodeId; 2],
    threshold: f32,
    grad: f32,
}

impl Diode {
    pub fn new() -> Diode {
        Diode {
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
    fn stamp_vector(&self, rhs_vec: &mut DVector<f32>, _: &DVector<f32>) {
        let g = self.threshold * self.grad;
        match self.pins {
            [p0, 0] => {
                let p0 = p0 - 1;
                rhs_vec[p0] += g;
            }
            [0, p1] => {
                let p1 = p1 - 1;
                rhs_vec[p1] -= g;
            }
            [p0, p1] => {
                let p0 = p0 - 1;
                let p1 = p1 - 1;
                rhs_vec[p0] += g;
                rhs_vec[p1] -= g;
            }
        }
    }

    fn stamp_matrix(&self, matrix: &mut DMatrix<f32>, state: &DVector<f32>) {
        match self.pins {
            [p0, 0] => {
                let p0 = p0 - 1;
                let didv = self.d_current(state[p0]);
                matrix[(p0, p0)] += didv;
            }
            [0, p1] => {
                let p1 = p1 - 1;
                let didv = self.d_current(state[p1]);
                matrix[(p1, p1)] += didv;
            }
            [p0, p1] => {
                let p0 = p0 - 1;
                let p1 = p1 - 1;
                let didv = self.d_current(state[p0] - state[p1]);
                matrix[(p0, p1)] -= didv;
                matrix[(p1, p0)] -= didv;
                matrix[(p0, p0)] += didv;
                matrix[(p1, p1)] += didv;
            }
        }
    }
}

#[derive(Debug)]
pub struct IndVoltageSrc {
    pins: [NodeId; 2],
    voltage: f32,
}

impl IndVoltageSrc {
    pub fn new(volt: f32) -> IndVoltageSrc {
        IndVoltageSrc {
            pins: [0, 0],
            voltage: volt,
        }
    }
}

impl Element for IndVoltageSrc {
    fn connect_pin_to_node(&mut self, pin_id: usize, node_id: usize) {
        self.pins[pin_id] = node_id;
    }

    fn is_voltage_src(&self) -> bool {
        true
    }

    fn stamp_vector_by_src(&self, vector: &mut DVector<f32>, index: usize) {
        vector[index] = self.voltage;
    }

    fn stamp_matrix_by_src(&self, matrix: &mut DMatrix<f32>, index: usize) {
        match self.pins {
            [p0, 0] => {
                let p0 = p0 - 1;
                matrix[(p0, index)] = 1.0;
                matrix[(index, p0)] = 1.0;
            }
            [0, p1] => {
                let p1 = p1 - 1;
                matrix[(p1, index)] = -1.0;
                matrix[(index, p1)] = -1.0;
            }
            [p0, p1] => {
                let p0 = p0 - 1;
                let p1 = p1 - 1;
                matrix[(p0, index)] = 1.0;
                matrix[(p1, index)] = -1.0;
                matrix[(index, p0)] = 1.0;
                matrix[(index, p1)] = -1.0;
            }
        }
    }
}
