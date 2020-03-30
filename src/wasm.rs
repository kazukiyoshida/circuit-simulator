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

    // 回路の概要
    //   ・以下の要素をまとめたもの. この概要から一意に詳細が定まる.
    //     ・回路素子の数や種類、内部パラメータ
    //     ・回路素子とノードの結合状態

    // 回路の詳細
    //   ・各ノードの電圧や電流.

    // アルゴリズム
    //   MCU が存在するならば clk を進める;
    //   回路の概要と以前のそれを比較する;
    //     1-a. 変化がある場合 => 回路の方程式を解き、回路の詳細を得る.
    //     1-a. 変化がない場合 => return None.
    //   以前の回路の詳細と比較して変化があったかどうかをみる;
    //     2-a. 変化があった場合   => return 回路の詳細.
    //     2-b. 変化がなかった場合 => return None.
    pub fn next(&mut self) -> String {
        // TODO
        // {
        //   1: 3,
        //   2: 2.3,
        //   3: 0.1,
        //   4: 0,
        // }: Record<node_id, voltage>
        "{1: 3, 2: 2.3, 3: 0.1, 4: 0}".to_string()
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

    // >>>> Arduino

    // Arduino を作成する
    pub fn add_arduino(&mut self) -> usize {
        0 // TODO
    }

    // Arduino にプログラムを書き込む
    pub fn arduino_program(&mut self, element_id: usize, hex: String) {
        // TODO
    }
}
