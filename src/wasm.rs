use super::simulator::*;
use std::cell::RefCell;
use std::rc::Rc;
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
pub struct Circuit(Simulator);

#[wasm_bindgen]
impl Circuit {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Circuit {
        Circuit(Simulator::new())
    }

    pub fn add_registor(&mut self, r: f32) -> usize {
        self.0.add_registor(r)
    }

    pub fn add_diode(&mut self) -> usize {
        self.0.add_diode()
    }

    pub fn add_ind_voltage_src(&mut self, v: f32) -> usize {
        self.0.add_ind_voltage_src(v)
    }

    pub fn add_node(&mut self) -> usize {
        self.0.add_node()
    }

    pub fn connect_elm_pin_node(
        &mut self, element_id: usize, pin_id: usize, node_id: usize)
    {
        self.0.connect_elm_pin_node(element_id, pin_id, node_id);
    }

    pub fn connect_elm_pin_gnd(&mut self, element_id: usize, pin_id: usize) {
        self.0.connect_elm_pin_gnd(element_id, pin_id);
    }
}
