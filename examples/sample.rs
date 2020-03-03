use circuit_simulator::circuit::*;
use circuit_simulator::elements::*;

fn main() {
    println!("\n>> Elements");

    let r1 = Resistor::new(
        "R1".to_string(),
        330_f32,
        [Some("N1".to_string()), Some("N2".to_string())],
    );
    println!("{:?}", r1);

    let r2 = Resistor::new(
        "R2".to_string(),
        20_f32,
        [Some("N2".to_string()), Some(GND.to_string())],
    );
    println!("{:?}", r2);

    let v = IndVoltageSrc {
        name: "V1".to_string(),
        v: 5_f32,
        nodes: [Some("N1".to_string()), Some(GND.to_string())],
    };
    println!("{:?}", v);

    let c = IndCurrentSrc {
        name: "C1".to_string(),
        i: 0.2_f32,
        nodes: [Some("N2".to_string()), Some(GND.to_string())],
    };
    println!("{:?}", c);

    let d = Diode::new(
        "Diode1".to_string(),
        [Some("N2".to_string()), Some(GND.to_string())],
    );
    println!("{:?}", d);

    println!("\n>> Circuit");

    let mut circuit = Circuit::new();
    circuit.add(Box::new(r1));
    // circuit.add(Box::new(r2));
    circuit.add(Box::new(v));
    // circuit.add(Box::new(c));
    circuit.add(Box::new(d));

    println!("{:#?}", &circuit);
    let v = circuit.solve_eq();
    if let Some(v) = v {
        println!("{}", v);
    }
}
