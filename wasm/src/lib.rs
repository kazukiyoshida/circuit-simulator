use avr_emulator::atmega328p::*;
use avr_emulator::avr::*;
use avr_emulator::logger::*;
use circuit_simulator::circuit::*;
use circuit_simulator::elements::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen(start)]
pub fn initialize() {
    set_panic_hook();
}

// エラー時により詳細なスタックトレースを表示
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub enum Element {
    Resistor,
    Diode,
    IndVoltageSrc,
    IndCurrentSrc,
}

#[wasm_bindgen]
pub struct CircuitController {
    circuit: Circuit,
}

#[wasm_bindgen]
impl CircuitController {
    #[wasm_bindgen(constructor)]
    pub fn new() -> CircuitController {
        let c = Circuit::new();
        CircuitController { circuit: c }
    }

    pub fn exec(&mut self, hex: String) {
        self.scenario1(hex);
    }
}

impl CircuitController {
    // シナリオ1: 5V 電源 - 抵抗 - LED - GND の定常回路
    pub fn scenario1(&mut self, hex: String) {
        // Setup circuit
        let v = IndVoltageSrc::new("V", 5f32, [Node::new("N1"), Node::gnd()]);
        let r = Resistor::new("R", 330f32, [Node::new("N1"), Node::new("N2")]);
        let d = Diode::new("LED", [Node::new("N2"), Node::gnd()]);
        self.circuit.add(Box::new(v));
        self.circuit.add(Box::new(r));
        self.circuit.add(Box::new(d));

        // Setup CPU
        let avr = ATmega328P::new();
        let mut timer0 = avr.new_timer0();
        let mut timer1 = avr.new_timer1();
        let mut timer2 = avr.new_timer2();
        let mut portb = avr.new_portb();
        let mut portc = avr.new_portc();
        let mut portd = avr.new_portd();
        let logger = Logger::new();

        avr.load_hex_from_string(hex);
        avr.initialize_sram();

        let mut last_pinb = 0;

        // CPU start
        loop {
            avr.execute();
            timer0.clk_io();
            timer1.clk_io();
            timer2.clk_io();
            portb.clk_io();
            portc.clk_io();
            portd.clk_io();

            if last_pinb != portb.pinx() {
                if bit(portb.pinx(), 5) {
                    if let Some(source) = self.circuit.elements[0].downcast_mut::<IndVoltageSrc>() {
                        source.v = 5f32;
                    }
                    console_log!("ON");
                } else {
                    if let Some(source) = self.circuit.elements[0].downcast_mut::<IndVoltageSrc>() {
                        source.v = 0f32;
                    }
                    console_log!("OFF");
                }
                last_pinb = portb.pinx();
            }
        }
    }
}

fn bit(a: u8, n: u8) -> bool {
    ((a & 1 << n) >> n) == 1
}
