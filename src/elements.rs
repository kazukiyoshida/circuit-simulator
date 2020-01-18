extern crate nalgebra;

use std::collections::BTreeSet;
use nalgebra::base::{DMatrix, DVector};
use super::circuit::*;


// 回路素子トレイト
pub trait Element: std::fmt::Debug {
    fn name(&self) -> &String;
    fn nodes(&self) -> Vec<&Node>;

    fn stamp_m(
        &self,
        _matrix: &mut DMatrix<f32>,
        _circuit: &Circuit,
        _state: &DVector<f32>
    ) {}

    fn stamp_v(
        &self,
        _vector: &mut DVector<f32>,
        _circuit: &Circuit
    ) {}

    fn is_voltage_source(&self) -> bool { false }
}


// 抵抗（抵抗値 (Ω)）
#[derive(Debug)]
pub struct Resistor {
    pub name: String,
    pub r: f32,
    pub nodes: [Node; 2],
}

impl Resistor {
    pub fn new(name: String, r: f32, nodes: [Node; 2]) -> Resistor {
        assert!(r > 0_f32);
        Resistor {
            name: name,
            r: r,
            nodes: nodes,
        }
    }

    fn g(&self) -> f32 { 1_f32/self.r }
}

impl Element for Resistor {
    fn name(&self) -> &String {
        &self.name
    }

    fn nodes(&self) -> Vec<&Node> {
        vec![&self.nodes[0], &self.nodes[1]]
    }

    fn stamp_m(&self, m: &mut DMatrix<f32>, circuit: &Circuit, _: &DVector<f32>) {
        // 自分のノード
        let n0 = self.nodes[0].as_ref().unwrap();
        let n1 = self.nodes[1].as_ref().unwrap();

        // 回路のノードを参照して行列のインデックスを決定
        let i0 = circuit.node_index(n0);
        let i1 = circuit.node_index(n1);

        if let (Some(i0_), Some(i1_)) = (i0, i1) {
            m[(i0_, i0_)] += self.g();
            m[(i1_, i1_)] += self.g();
            m[(i0_, i1_)] -= self.g();
            m[(i1_, i0_)] -= self.g();
        } else {
            if let Some(i) = i0 {
                m[(i, i)] += self.g();
            }
            if let Some(i) = i1 {
                m[(i, i)] += self.g();
            }
        }
    }
}

// // キャパシタ（容量値 (F)）
// #[derive(Debug)]
// pub struct Capacitor {
//     pub name: String,
//     pub f: f32,
// }
//
// impl Element for Capacitor {
//     fn nodes(&self) -> Vec<&Node> {
//         vec![]
//     }
// }

// ダイオード
#[derive(Debug)]
pub struct Diode {
    pub name: String,
    pub nodes: [Node; 2],
}

impl Element for Diode {
    fn name(&self) -> &String {
        &self.name
    }

    fn nodes(&self) -> Vec<&Node> {
        vec![&self.nodes[0], &self.nodes[1]]
    }
}

// // バイポーラ接合型トランジスタ
// #[derive(Debug)]
// pub struct BJT {
//     pub name: String,
// }
//
// impl Element for BJT {
//     fn nodes(&self) -> Vec<&Node> {
//         vec![]
//     }
// }
//
// // MOS 型電界効果トランジスタ
// #[derive(Debug)]
// pub struct MOSFET {
//     pub name: String,
// }
//
// impl Element for MOSFET {
//     fn nodes(&self) -> Vec<&Node> {
//         vec![]
//     }
// }

// 独立電圧源（電圧値(V)）
#[derive(Debug)]
pub struct IndependentVoltageSource {
    pub name: String,
    pub v: f32,
    pub nodes: [Node; 2],
}

impl Element for IndependentVoltageSource {
    fn name(&self) -> &String {
        &self.name
    }

    fn nodes(&self) -> Vec<&Node> {
        vec![&self.nodes[0], &self.nodes[1]]
    }

    fn stamp_m(
        &self,
        m: &mut DMatrix<f32>,
        circuit: &Circuit,
        _: &DVector<f32>,
    ) {
        // 自分のノード
        let n0 = self.nodes[0].as_ref().unwrap();
        let n1 = self.nodes[1].as_ref().unwrap();

        // 回路のノード・電圧源を参照して行列のインデックスを決定
        let i0 = circuit.node_index(n0);
        let i1 = circuit.node_index(n1);
        let iv = circuit.voltage_index(&self.name);
        let iv = circuit.nodes.len() + iv;

        if let (Some(i0_), Some(i1_)) = (i0, i1) {
            m[(i0_, iv)] = 1_f32;
            m[(i1_, iv)] = -1_f32;
            m[(iv, i0_)] = 1_f32;
            m[(iv, i1_)] = -1_f32;
        } else {
            if let Some(i) = i0 {
                m[(i, iv)] = 1_f32;
                m[(iv, i)] = 1_f32;
            }
            if let Some(i) = i1 {
                m[(i, iv)] = 1_f32;
                m[(iv, i)] = 1_f32;
            }
        }
    }

    fn stamp_v(&self, vec: &mut DVector<f32>, circuit: &Circuit) {
        let iv = circuit.voltage_index(&self.name);
        let iv = circuit.nodes.len() + iv;
        vec[iv] = self.v;
    }

    fn is_voltage_source(&self) -> bool { true }
}

// 独立電流源（電流値(A)）
#[derive(Debug)]
pub struct IndependentCurrentSource {
    pub name: String,
    pub i: f32,
    pub nodes: [Node; 2],
}

impl Element for IndependentCurrentSource {
    fn name(&self) -> &String {
        &self.name
    }

    fn nodes(&self) -> Vec<&Node> {
        vec![&self.nodes[0], &self.nodes[1]]
    }

    fn stamp_v(&self, vec: &mut DVector<f32>, circuit: &Circuit) {
        // 自分のノード
        let n0 = self.nodes[0].as_ref().unwrap();
        let n1 = self.nodes[1].as_ref().unwrap();

        // 回路のノードを参照して行列のインデックスを決定
        let i0 = circuit.node_index(n0);
        let i1 = circuit.node_index(n1);

        if let Some(i) = i0 {
            vec[i] += self.i;
        }
        if let Some(i) = i1 {
            vec[i] -= self.i;
        }
    }
}

