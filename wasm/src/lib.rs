#[macro_use]
extern crate downcast_rs;
use downcast_rs::DowncastSync;

use wasm_bindgen::prelude::*;
use circuit_simulator::circuit::*;
use circuit_simulator::elements::*;
use avr_emulator::atmega328p::*;
use avr_emulator::avr::*;

pub const SAMPLE_FILE_NAME: &str = "hex/atmel_studio/led_flashing_fast/led_flashing.hex";

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
        CircuitController {
            circuit: c,
        }
    }

    pub fn exec_test(&mut self) {
        self.scenario1();
    }
}

impl CircuitController {

    // シナリオテスト1.
    // 5V 電源 - 抵抗 - LED - GND の定常回路
    pub fn scenario1(&mut self) {
        // let ten_millis = time::Duration::from_millis(100);

        // 回路の作成
        let v = IndVoltageSrc {
            name: "V".to_string(),
            v: 5_f32,
            nodes: [Some("N1".to_string()), Some(GND.to_string())]
        };
        let r = Resistor::new(
            "R".to_string(),
            330_f32,
            [Some("N1".to_string()), Some("N2".to_string())]
        );
        let d = Diode::new(
            "LED".to_string(),
            [Some("N2".to_string()), Some(GND.to_string())]
        );
        self.circuit.add(Box::new(v));
        self.circuit.add(Box::new(r));
        self.circuit.add(Box::new(d));

        println!("|||||||||||||||||||");
        // CPU の準備
        let avr = ATmega328P::new();

        let mut timer0 = avr.new_timer0();
        let mut timer1 = avr.new_timer1();
        let mut timer2 = avr.new_timer2();
        let mut portb = avr.new_portb();
        let mut portc = avr.new_portc();
        let mut portd = avr.new_portd();

        avr.load_hex(SAMPLE_FILE_NAME);
        avr.initialize_sram();

        let mut last_pinb = 0;

        // CPU 起動
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
                } else {
                    if let Some(source) = self.circuit.elements[0].downcast_mut::<IndVoltageSrc>() {
                        source.v = 0f32;
                    }
                }

                println!("-----------------------------------");
                println!("{:?}", self.circuit);

                // IndVoltageSrc


                last_pinb = portb.pinx()
            }

            // if avr.cycle() % 1000 == 0 {
            //     println!("cycle = {:10}    PORTB = {}", avr.cycle(), portb,);
            // }
        }
    }

}

#[test]
fn test_scenario1() {
    let mut c = CircuitController::new();
    c.scenario1();
}

pub fn bit(a: u8, n: u8) -> bool {
    ((a & 1 << n) >> n) == 1
}

