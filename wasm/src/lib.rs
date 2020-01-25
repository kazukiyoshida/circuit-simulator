extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

extern crate circuit_simulator;
use circuit_simulator::circuit::*;
use circuit_simulator::elements::*;

#[wasm_bindgen]
pub enum Element {
  Resistor,
  Diode,
  IndependentVoltageSource,
  IndependentCurrentSource,
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

    pub fn exec_test(&mut self) -> String {
        self.scenario_test()
    }
}

impl CircuitController {
    // シナリオテスト
    pub fn scenario_test(&mut self) -> String {
        let r = Resistor::new(
            "R1".to_string(),
            330_f32,
            [Some("N1".to_string()), Some("N2".to_string())]
        );
        let v = IndependentVoltageSource {
            name: "V1".to_string(),
            v: 5_f32,
            nodes: [Some("N1".to_string()), Some(GND.to_string())]
        };
        let d = Diode::new(
            "Diode1".to_string(),
            [Some("N2".to_string()), Some(GND.to_string())]
        );

        self.circuit.add(Box::new(r));
        self.circuit.add(Box::new(v));
        self.circuit.add(Box::new(d));

        let vec = self.circuit.solve_eq();
        if let Some(vec) = vec {
            format!("{}", vec)
        } else {
            "error!".to_string()
        }
    }
}
