use downcast_rs::DowncastSync;
use nalgebra::base::{DMatrix, DVector};
use std::cell::RefCell;
use std::rc::Rc;

// 各要素の参照関係
// Element 1 : N Pin M : 1 Node

// Node への参照
type NodeId = usize;

pub trait Element: std::fmt::Debug + DowncastSync {
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
    pins: [Option<NodeId>; 2],
    resistance: f32,
}

impl Registor {
    pub fn new(registance: f32) -> Registor {
        let r = if registance == 0.0 { 0.01 } else { registance };
        Registor {
            pins: [None, None],
            resistance: r,
        }
    }

    pub fn conductance(&self) -> f32 {
        1.0 / self.resistance
    }
}

impl Element for Registor {
    fn connect_pin_to_node(&mut self, pin_id: usize, node_id: usize) {
        self.pins[pin_id] = Some(node_id);
    }

    fn stamp_matrix(&self, matrix: &mut DMatrix<f32>, _: &DVector<f32>) {
        // shift 1 for GND ...
        let p0 = self.pins[0].and_then(|n| n.checked_sub(1usize));
        let p1 = self.pins[1].and_then(|n| n.checked_sub(1usize));
        match [p0, p1] {
            [Some(p0), Some(p1)] => {
                matrix[(p0, p0)] += self.conductance();
                matrix[(p1, p1)] += self.conductance();
                matrix[(p0, p1)] -= self.conductance();
                matrix[(p1, p0)] -= self.conductance();
            }
            [Some(p0), None] => matrix[(p0, p0)] += self.conductance(),
            [None, Some(p1)] => matrix[(p1, p1)] += self.conductance(),
            [None, None] => {}
        };
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
    pins: [Option<NodeId>; 2],
    threshold: f32,
    grad: f32,
}

impl Diode {
    pub fn new() -> Diode {
        Diode {
            pins: [None, None],
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
        self.pins[pin_id] = Some(node_id);
    }

    // cf. https://spicesharp.github.io/SpiceSharp/articles/custom_components/modified_nodal_analysis.html
    fn stamp_vector(&self, rhs_vec: &mut DVector<f32>, state: &DVector<f32>) {
        // shift 1 for GND ...
        let p0 = self.pins[0].and_then(|n| n.checked_sub(1usize)); // anode
        let p1 = self.pins[1].and_then(|n| n.checked_sub(1usize)); // cathode

        match [p0, p1] {
            [Some(p0), Some(p1)] => {
                // let didv = self.d_current(state[p0] - state[p1]);
                // println!("##### state[p0]  : {}", state[p0]);
                // println!("##### state[p1]  : {}", state[p1]);
                // println!("##### d_current  : {}", didv);
                // matrix[(p0, p0)] += didv;
                // matrix[(p1, p1)] += didv;
                // matrix[(p0, p1)] -= didv;
                // matrix[(p1, p0)] -= didv;
            }
            [Some(p0), None] => {
                if state[p0] >= self.threshold {
                    rhs_vec[p0] += self.threshold * self.grad;
                }
            }
            [None, _] => {}
        };
    }

    fn stamp_matrix(&self, matrix: &mut DMatrix<f32>, state: &DVector<f32>) {
        // shift 1 for GND ...
        let p0 = self.pins[0].and_then(|n| n.checked_sub(1usize)); // anode
        let p1 = self.pins[1].and_then(|n| n.checked_sub(1usize)); // cathode
        match [p0, p1] {
            [Some(p0), Some(p1)] => {
                let didv = self.d_current(state[p0] - state[p1]);
                matrix[(p0, p0)] += didv;
                matrix[(p1, p1)] += didv;
                matrix[(p0, p1)] -= didv;
                matrix[(p1, p0)] -= didv;
            }
            [Some(p0), None] => {
                let didv = self.d_current(state[p0]);
                matrix[(p0, p0)] += didv;
            }
            [None, _] => {}
        };
    }
}

#[derive(Debug)]
pub struct IndVoltageSrc {
    pins: [Option<NodeId>; 2],
    voltage: f32,
}

impl IndVoltageSrc {
    pub fn new(volt: f32) -> IndVoltageSrc {
        IndVoltageSrc {
            pins: [None, None],
            voltage: volt,
        }
    }
}

impl Element for IndVoltageSrc {
    fn connect_pin_to_node(&mut self, pin_id: usize, node_id: usize) {
        self.pins[pin_id] = Some(node_id);
    }

    fn is_voltage_src(&self) -> bool {
        true
    }

    fn stamp_vector_by_src(&self, vector: &mut DVector<f32>, index: usize) {
        vector[index] = self.voltage;
    }

    fn stamp_matrix_by_src(&self, matrix: &mut DMatrix<f32>, index: usize) {
        // shift 1 for GND ...
        let p0 = self.pins[0].and_then(|n| n.checked_sub(1usize));
        let p1 = self.pins[1].and_then(|n| n.checked_sub(1usize));
        match [p0, p1] {
            [Some(p0), Some(p1)] => {
                matrix[(p0, index)] = 1.0;
                matrix[(p1, index)] = -1.0;
                matrix[(index, p0)] = 1.0;
                matrix[(index, p1)] = -1.0;
            }
            [Some(p0), None] => {
                matrix[(p0, index)] = 1.0;
                matrix[(index, p0)] = 1.0;
            }
            [None, Some(p1)] => {
                matrix[(p1, index)] = 1.0;
                matrix[(index, p1)] = 1.0;
            }
            [None, None] => {}
        };
    }
}
