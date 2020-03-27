use super::link::*;
use nalgebra::base::{DMatrix, DVector};
use std::cell::{Cell, RefCell};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug)]
pub struct Registor {
    id: usize,
    pub pins: u8,
    pub resistance: f32,
    pub is_voltage_src: bool,
}

#[wasm_bindgen]
impl Registor {
    #[wasm_bindgen(constructor)]
    pub fn new(registance: f32) -> Rc<RefCell<Registor>> {
        let r = if registance == 0.0 { 0.01 } else { registance };
        let registor = Registor {
            id: get_element_id(),
            pins: 2,
            resistance: r,
            is_voltage_src: false,
        };
        Rc::new(RefCell::new(registor))
    }

    fn conductance(&self) -> f32 {
        1.0 / self.resistance
    }

    pub fn connect_pin_to_node(&mut self, pin_id: usize, node_id: usize) {
        LINKS.with(|links| {
            links.borrow_mut().add(self.id, pin_id, node_id);
        });
    }

    pub fn stamp(&self) {
        //     match self.pins {
        //         [p0, 0] => {
        //             let p0 = p0 - 1;
        //             matrix[(p0, p0)] += self.conductance();
        //         }
        //         [0, p1] => {
        //             let p1 = p1 - 1;
        //             matrix[(p1, p1)] += self.conductance();
        //         }
        //         [p0, p1] => {
        //             let p0 = p0 - 1;
        //             let p1 = p1 - 1;
        //             matrix[(p0, p0)] += self.conductance();
        //             matrix[(p1, p1)] += self.conductance();
        //             matrix[(p0, p1)] -= self.conductance();
        //             matrix[(p1, p0)] -= self.conductance();
        //         }
        //     }
    }
}

// ダイオード
// 順方向電圧 Vd における電流 I(Vd) を区分線形近似でモデリングする.
// I(V) = 0                      ( Vd <= threshold )
//      = grad * (V - threshold) ( Vd  > threshold )
#[wasm_bindgen]
#[derive(Debug)]
pub struct Diode {
    id: usize,
    // pins[0] : Anode
    // pins[1] : Cathode
    pub pins: u8,
    pub threshold: f32,
    pub grad: f32,
    pub is_voltage_src: bool,
}

#[wasm_bindgen]
impl Diode {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Diode {
        Diode {
            id: get_element_id(),
            pins: 2,
            threshold: 0.674,
            grad: 0.191,
            is_voltage_src: false,
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

    pub fn connect_pin_to_node(&mut self, pin_id: usize, node_id: usize) {
        LINKS.with(|links| {
            links.borrow_mut().add(self.id, pin_id, node_id);
        });
    }

    pub fn stamp(&self) {
        //     match self.pins {
        //         [p0, 0] => {
        //             let p0 = p0 - 1;
        //             let didv = self.d_current(state[p0]);
        //             matrix[(p0, p0)] += didv;
        //         }
        //         [0, p1] => {
        //             let p1 = p1 - 1;
        //             let didv = self.d_current(state[p1]);
        //             matrix[(p1, p1)] += didv;
        //         }
        //         [p0, p1] => {
        //             let p0 = p0 - 1;
        //             let p1 = p1 - 1;
        //             let didv = self.d_current(state[p0] - state[p1]);
        //             matrix[(p0, p1)] -= didv;
        //             matrix[(p1, p0)] -= didv;
        //             matrix[(p0, p0)] += didv;
        //             matrix[(p1, p1)] += didv;
        //         }
        //     }

        //     let g = self.threshold * self.grad;
        //     match self.pins {
        //         [p0, 0] => {
        //             let p0 = p0 - 1;
        //             rhs_vec[p0] += g;
        //         }
        //         [0, p1] => {
        //             let p1 = p1 - 1;
        //             rhs_vec[p1] -= g;
        //         }
        //         [p0, p1] => {
        //             let p0 = p0 - 1;
        //             let p1 = p1 - 1;
        //             rhs_vec[p0] += g;
        //             rhs_vec[p1] -= g;
        //         }
        //     }
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct IndVoltageSrc {
    id: usize,
    pub pins: u8,
    pub voltage: f32,
    pub is_voltage_src: bool,
}

#[wasm_bindgen]
impl IndVoltageSrc {
    #[wasm_bindgen(constructor)]
    pub fn new(volt: f32) -> IndVoltageSrc {
        IndVoltageSrc {
            id: get_element_id(),
            pins: 2,
            voltage: volt,
            is_voltage_src: true,
        }
    }
}

#[wasm_bindgen]
impl IndVoltageSrc {
    pub fn connect_pin_to_node(&mut self, pin_id: usize, node_id: usize) {
        LINKS.with(|links| {
            links.borrow_mut().add(self.id, pin_id, node_id);
        });
    }

    pub fn stamp(&self) {
        //     vector[index] = self.voltage;

        //     match self.pins {
        //         [p0, 0] => {
        //             let p0 = p0 - 1;
        //             matrix[(p0, index)] = 1.0;
        //             matrix[(index, p0)] = 1.0;
        //         }
        //         [0, p1] => {
        //             let p1 = p1 - 1;
        //             matrix[(p1, index)] = -1.0;
        //             matrix[(index, p1)] = -1.0;
        //         }
        //         [p0, p1] => {
        //             let p0 = p0 - 1;
        //             let p1 = p1 - 1;
        //             matrix[(p0, index)] = 1.0;
        //             matrix[(p1, index)] = -1.0;
        //             matrix[(index, p0)] = 1.0;
        //             matrix[(index, p1)] = -1.0;
        //         }
        //     }
    }
}

thread_local! {
    static ELEMENT_COUNT: Cell<usize> = Cell::new(0);
}

fn get_element_id() -> usize {
    ELEMENT_COUNT.with(|element_no| {
        let id = element_no.get();
        element_no.set(id + 1);
        id
    })
}
