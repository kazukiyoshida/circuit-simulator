use super::super::simulator::*;
use super::element::*;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[derive(Debug)]
pub struct IndVoltageSrc {
    id: usize,
    pins: [usize; 2],
    outputs: [bool; 2],
    voltage: f32,
}

impl IndVoltageSrc {
    pub fn new(id: usize, volt: f32) -> IndVoltageSrc {
        IndVoltageSrc {
            id: id,
            pins: [0, 0],
            outputs: [true, false],
            voltage: volt,
        }
    }
}

impl Element for IndVoltageSrc {
    fn connect_pin_to_node(&mut self, pin_id: usize, node_id: usize) {
        self.pins[pin_id] = node_id;
    }

    fn output_pins(&self) -> Vec<bool> {
        self.outputs.to_vec()
    }

    fn stamp(&self, eq: &mut Equation) {
        // 出力ピンは 0 だけ
        let index = eq.src_index.get(&(self.id, 0)).unwrap() + eq.node_index.len();
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

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl Simulator {
    pub fn add_ind_voltage_src(&mut self, v: f32) -> usize {
        let id = self.elements.keys().max().unwrap_or(&0usize) + 1;
        let element = Rc::new(RefCell::new(IndVoltageSrc::new(id, v)));
        self.elements.insert(id, element);
        id
    }
}
