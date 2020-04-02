use super::simulator::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Circuit(Simulator);

#[wasm_bindgen]
impl Circuit {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Circuit {
        Circuit(Simulator::new())
    }

    // 回路の詳細を求める（定常状態を計算する）
    pub fn calc(&mut self) -> String {
        "{1: 3, 2: 2.3, 3: 0.1, 4: 0}".to_string()
    }

    // 回路の詳細を求める（非定常状態を計算する）
    pub fn next(&mut self) -> Option<String> {
        // Record<node_id, voltage>
        Some("{1: 3, 2: 2.3, 3: 0.1, 4: 0}".to_string())
    }

    //--------------------------------------------------------------------------
    // ノード

    // ノードを作成する
    pub fn add_node(&mut self) -> usize {
        self.0.add_node()
    }

    // 回路素子の端子をノードに接続する
    pub fn connect_element_pin_node(&mut self, element_id: usize, pin_id: usize, node_id: usize) {
        self.0.connect_element_pin_node(element_id, pin_id, node_id);
    }

    // 回路素子の端子をノードに接続する
    pub fn connect_elment_pin_gnd(&mut self, element_id: usize, pin_id: usize) {
        self.0.connect_elment_pin_gnd(element_id, pin_id);
    }

    //--------------------------------------------------------------------------
    // 回路素子

    // >>>> 電気抵抗

    // 電気抵抗を作成する
    pub fn add_registor(&mut self, r: f32) -> usize {
        self.0.add_registor(r)
    }

    // 電気抵抗の抵抗値を変化させる
    pub fn registor_change_registance(&mut self, element_id: usize, r: f32) {
        self.0.registor_change_registance(element_id, r);
    }

    // >>>> ダイオード

    // ダイオードを作成する
    pub fn add_diode(&mut self) -> usize {
        self.0.add_diode()
    }

    // >>>> 定常電圧源

    // 定常電圧源を作成する
    pub fn add_ind_voltage_src(&mut self, v: f32) -> usize {
        self.0.add_ind_voltage_src(v)
    }

    // >>>> ArduinoUno

    // ArduinoUno を作成する
    pub fn add_arduino_uno(&mut self) -> usize {
        self.0.add_arduino_uno()
    }

    // ArduinoUno にプログラムを書き込む
    pub fn arduino_uno_program(&mut self, element_id: usize, hex: String) {
        self.0.arduino_uno_program(element_id, hex);
    }
}
