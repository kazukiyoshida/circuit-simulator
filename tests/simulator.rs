use circuit_simulator::simulator::*;
use std::fs;

#[test]
fn test_simulator_gnd_v_r_led_gnd() {
    let mut sim = Simulator::new();

    // 電源 - N1 - 抵抗 - N2 - LED - GND
    let eid0 = sim.add_ind_voltage_src(5.0);
    let eid1 = sim.add_registor(330.0);
    let eid2 = sim.add_diode();

    let node0 = sim.add_node();
    let node1 = sim.add_node();

    sim.connect_element_pin_node(eid0, 0, node0);
    sim.connect_element_pin_node(eid1, 0, node0);
    sim.connect_element_pin_node(eid1, 1, node1);
    sim.connect_element_pin_node(eid2, 0, node1);

    match sim.state() {
        Ok(state) => println!(" state : {:?}", state),
        Err(err) => (),
    };
}

#[test]
fn test_simulator_gnd_v() {
    let mut sim = Simulator::new();

    // GND - 電源 - N1  （開放）
    let eid0 = sim.add_ind_voltage_src(5.0);
    let node0 = sim.add_node();
    sim.connect_element_pin_node(eid0, 0, node0);

    match sim.state() {
        Ok(state) => println!(" state : {:?}", state),
        Err(err) => (),
    };
}

#[test]
fn test_simulator_gnd_v_r() {
    let mut sim = Simulator::new();

    // GND - 電源 - N1 - 抵抗 - N2  （開放）
    let eid0 = sim.add_ind_voltage_src(5.0);
    let eid1 = sim.add_registor(330.0);

    let node0 = sim.add_node();
    let node1 = sim.add_node();

    sim.connect_element_pin_node(eid0, 0, node0);
    sim.connect_element_pin_node(eid1, 0, node0);
    sim.connect_element_pin_node(eid1, 1, node1);

    match sim.state() {
        Ok(state) => println!(" state : {:?}", state),
        Err(err) => (),
    };
}

#[test]
fn test_simulator_gnd_v_r_gnd() {
    let mut sim = Simulator::new();

    // GND - 電源 - N1 - 抵抗 - GND
    let eid0 = sim.add_ind_voltage_src(5.0);
    let eid1 = sim.add_registor(5.0);

    let node0 = sim.add_node();

    sim.connect_element_pin_node(eid0, 0, node0);
    sim.connect_element_pin_node(eid1, 0, node0);

    match sim.state() {
        Ok(state) => println!(" state : {:?}", state),
        Err(err) => (),
    };
}

const SAMPLE_FILE_NAME: &str = "tests/hex/led_flashing.hex";

#[test]
fn test_simulator_arduinouno() {
    let mut sim = Simulator::new();

    // ArduinoUno - N1 - 抵抗 - N2 - LED - GND
    let eid0 = sim.add_arduino_uno();
    let eid1 = sim.add_registor(330.0);
    let eid2 = sim.add_diode();

    let node0 = sim.add_node();
    let node1 = sim.add_node();

    sim.connect_element_pin_node(eid0, 18, node0);
    sim.connect_element_pin_node(eid1, 0, node0);
    sim.connect_element_pin_node(eid1, 1, node1);
    sim.connect_element_pin_node(eid2, 0, node1);

    let hex = fs::read_to_string(SAMPLE_FILE_NAME).unwrap();
    sim.arduino_uno_program(eid0, hex);

    sim.update_state();

    loop {
        match sim.next() {
            Ok(maybeState) => match maybeState {
                Some(state) => println!("||| state : {:?}", state),
                None => (),
            },
            Err(err) => println!("||| err : {}", err),
        };
    }
}
