extern crate nalgebra;
extern crate circuit_simulator;
extern crate avr_emulator as avr;

use std::{thread, time};
use circuit_simulator::circuit::*;
use circuit_simulator::elements::*;

fn main() {
    let ten_millis = time::Duration::from_millis(100);
    let now = time::Instant::now();

    // cpu 作成
    let mut core = avr::core::Core::new();

    //_プログラム読み込み
    println!("--- load hex file ---");
    core.load_hex("../avr-emulator/src/bin/sample.hex");
    println!("Mem");
    println!("{}", core.mem);

    // Arduino 基盤（Circuit） 作成
    //     （今の所は）CircuitSimulator リポジトリに寄せて、基盤を作成する.

    // 基盤の初期状態を計算
    //     この時点で回路の状態が定まり、画面に描画が可能になる.

    // CPU 起動
    //     この loop は CPU オブジェクトに寄せた方が良い
    println!("--- start ---");
    loop {
        // 第 n サイクル時の状態を確認
        println!("\n|||| core ||||");
        println!("Cycles:         {:?}", core.cycles);
        println!("SP:             {}",   core.sp());
        println!("Registers:      {:?}", core.regs());
        println!("StatusRegister: {:?}", core.sreg());

        // 回路の状態を更新
        //     出力ポートの flg が立っていたらそのポートを 5V とみなす

        // 回路の方程式を解く
        //     この時点で回路の状態が定まり、画面に描画が可能になる.
        //     キャッシュが効くならば計算不要.

        core.next();
        thread::sleep(ten_millis);
    }
}
