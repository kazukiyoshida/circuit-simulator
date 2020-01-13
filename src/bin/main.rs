extern crate circuit_simulator;
extern crate avr_emulator;
extern crate ndarray;

use circuit_simulator::*;
use ndarray::prelude::*;

fn main() {
    // avr_emulator との接続チェック
    println!("|||||||||||| avr_emulator |||||||||||||");
    avr_emulator::hello();

    println!("|||||||||||| circuit simulator ||||||||||||");
    let r1 = Resistor{
        name: "R1".to_string(),
        r: 10,
        nodes: [Some("N0".to_string()), Some("N1".to_string())]
    };
    println!("{:?}", r1);
    let r2 = Resistor{
        name: "R2".to_string(),
        r: 10,
        nodes: [None, None]
    };
    println!("{:?}", r2);

    let mut circuit = Circuit::new();
    println!("{:?}", circuit);
    circuit.add(Box::new(r1));
    circuit.add(Box::new(r2));
    println!("{:?}", circuit);
    // println!("{:?}", r1); // borrow of moved value


    println!("|||||||||||| ndarray ||||||||||||");
    let a = Array::<f64, _>::zeros(3);
    let b = Array::range(0., 9., 1.).into_shape((3, 3)).unwrap();
    let c = arr2(&[
        [3.0, 1.0, 1.0],
        [1.0, 3.0, 1.0],
        [1.0, 1.0, 3.0]
    ]);
    println!("{}", a);
    println!("{}", b);
    println!("{}", c);
}
