use super::super::simulator::*;
use super::element::*;
use avr_emulator::arch::atmega328p::*;
use avr_emulator::avrmcu::AVRMCU;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

pub struct ArduinoUno {
    id: usize,
    pins: [usize; 20],
    avr: Rc<RefCell<ATmega328P>>,
}

impl ArduinoUno {
    pub fn new(id: usize) -> ArduinoUno {
        ArduinoUno {
            id: id,
            pins: [0; 20],
            avr: Rc::new(RefCell::new(ATmega328P::new(Package::PDIP28))),
        }
    }

    pub fn program(&self, hex: String) {
        self.avr.borrow().program(hex);
        self.avr.borrow_mut().initialize();
    }
}

impl Element for ArduinoUno {
    fn connect_pin_to_node(&mut self, pin_id: usize, node_id: usize) {
        self.pins[pin_id] = node_id;
    }

    fn stamp(&self, eq: &mut Equation) {
        // TODO
    }

    fn clk(&self) -> bool {
        let mut avr = self.avr.borrow_mut();
        let pins = avr.get_pins();
        avr.next();
        pins != avr.get_pins()
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl Simulator {
    pub fn add_arduino_uno(&mut self) -> usize {
        let id = self.elements.keys().max().unwrap_or(&0usize) + 1;
        let element = Rc::new(RefCell::new(ArduinoUno::new(id)));
        self.elements.insert(id, element);
        id
    }

    pub fn arduino_uno_program(&mut self, element_id: usize, hex: String) {
        match self
            .elements
            .get(&element_id)
            .unwrap()
            .borrow_mut()
            .as_any()
            .downcast_mut::<ArduinoUno>()
        {
            Some(arduino) => arduino.program(hex),
            None => panic!("is not ArduinoUno"),
        }
    }
}
