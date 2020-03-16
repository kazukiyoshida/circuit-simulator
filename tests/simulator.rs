use circuit_simulator::simulator::*;

#[test]
fn test_simulator_gnd_v_r_led_gnd() {
    let mut sim = Simulator::new();

    // 電源 - N1 - 抵抗 - N2 - LED - GND
    let eid0 = sim.add_ind_voltage_src(5.0);
    let eid1 = sim.add_registor(330.0);
    let eid2 = sim.add_diode();

    let node0 = sim.add_node();
    let node1 = sim.add_node();

    sim.connect_pin_and_node(sim.pin_ref(eid0, 0), node0);
    sim.connect_pin_and_node(sim.pin_ref(eid1, 0), node0);
    sim.connect_pin_and_node(sim.pin_ref(eid1, 1), node1);
    sim.connect_pin_and_node(sim.pin_ref(eid2, 0), node1);
    sim.connect_pin_and_gnd(sim.pin_ref(eid0, 1));
    sim.connect_pin_and_gnd(sim.pin_ref(eid2, 1));

    match &sim.solve_eq() {
        Some(vector) => println!("result ... \n {}", vector),
        None => {}
    }
}

#[test]
fn test_simulator_gnd_v() {
    let mut sim = Simulator::new();

    // GND - 電源 - N1  （開放）
    let eid0 = sim.add_ind_voltage_src(5.0);

    let node0 = sim.add_node();

    sim.connect_pin_and_node(sim.pin_ref(eid0, 0), node0);
    sim.connect_pin_and_gnd(sim.pin_ref(eid0, 1));

    match &sim.solve_eq() {
        Some(vector) => println!("result ... \n {}", vector),
        None => {}
    }
}

#[test]
fn test_simulator_gnd_v_r() {
    let mut sim = Simulator::new();

    // GND - 電源 - N1 - 抵抗 - N2  （開放）
    let eid0 = sim.add_ind_voltage_src(5.0);
    let eid1 = sim.add_registor(330.0);

    let node0 = sim.add_node();
    let node1 = sim.add_node();

    sim.connect_pin_and_node(sim.pin_ref(eid0, 0), node0);
    sim.connect_pin_and_node(sim.pin_ref(eid1, 0), node0);
    sim.connect_pin_and_node(sim.pin_ref(eid1, 1), node1);
    sim.connect_pin_and_gnd(sim.pin_ref(eid0, 1));

    match &sim.solve_eq() {
        Some(vector) => println!("result ... \n {}", vector),
        None => {}
    }
}

#[test]
fn test_simulator_gnd_v_r_gnd() {
    let mut sim = Simulator::new();

    // GND - 電源 - N1 - 抵抗 - GND
    let eid0 = sim.add_ind_voltage_src(5.0);
    let eid1 = sim.add_registor(5.0);

    let node0 = sim.add_node();

    sim.connect_pin_and_node(sim.pin_ref(eid0, 0), node0);
    sim.connect_pin_and_node(sim.pin_ref(eid1, 0), node0);
    sim.connect_pin_and_gnd(sim.pin_ref(eid0, 1));
    sim.connect_pin_and_gnd(sim.pin_ref(eid1, 1));

    match &sim.solve_eq() {
        Some(vector) => println!("result ... \n {}", vector),
        None => {}
    }
}
