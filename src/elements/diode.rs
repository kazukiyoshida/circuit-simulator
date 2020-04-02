use super::super::simulator::*;
use super::element::*;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

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

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl Simulator {
    pub fn add_diode(&mut self) -> usize {
        let id = self.elements.keys().max().unwrap_or(&0usize) + 1;
        let element = Rc::new(RefCell::new(Diode::new(id)));
        self.elements.insert(id, element);
        id
    }
}
