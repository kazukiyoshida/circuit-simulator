use circuit_simulator::circuit::*;
use circuit_simulator::elements::*;

fn main() {
    let r1 = Resistor::new("R1", 330_f32, [Node::new("N1"), Node::new("N2")]);
    let r2 = Resistor::new("R2", 20_f32, [Node::new("N2"), Node::gnd()]);
    let v = IndVoltageSrc::new("V1", 5_f32, [Node::new("N1"), Node::gnd()]);
    let c = IndCurrentSrc::new("C1", 0.2_f32, [Node::new("N2"), Node::gnd()]);
    let d = Diode::new("Diode1", [Node::new("N2"), Node::gnd()]);

    println!("\n>> Elements");
    println!("{:?}", r1);
    println!("{:?}", r2);
    println!("{:?}", v);
    println!("{:?}", c);
    println!("{:?}", d);

    let mut circuit = Circuit::new();
    circuit.add(Box::new(r1));
    circuit.add(Box::new(v));
    circuit.add(Box::new(d));

    println!("\n>> Circuit");
    println!("{:#?}", &circuit);

    let v = circuit.solve_eq();
    if let Some(v) = v {
        println!("{}", v);
    }
}
