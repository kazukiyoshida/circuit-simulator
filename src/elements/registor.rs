use super::element::*;
use super::super::simulator::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::any::Any;
use wasm_bindgen::prelude::*;

#[derive(Debug)]
pub struct Registor {
    id: usize,
    // 結合しているノードの id. デフォルトでは GND に結合している.
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

    fn change_registance(&mut self, r: f32) {
        self.resistance = r;
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

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl Simulator {
    pub fn add_registor(&mut self, r: f32) -> usize {
        let id = self.elements.keys().max().unwrap_or(&0usize) + 1;
        let element = Rc::new(RefCell::new(Registor::new(id, r)));
        self.elements.insert(id, element);
        id
    }

    pub fn registor_change_registance(&mut self, element_id: usize, r: f32) {
        match self
            .elements
            .get(&element_id)
            .unwrap()
            .borrow_mut()
            .as_any()
            .downcast_mut::<Registor>()
        {
            Some(registor) => registor.resistance = r,
            None => panic!("is not Registor"),
        }
    }
}

