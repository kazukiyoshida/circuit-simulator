extern crate nalgebra as na;

use std::collections::BTreeSet;
use nalgebra::base::{DMatrix, DVector};


pub trait Element: std::fmt::Debug {
    fn name(&self) -> &String;
    fn nodes(&self) -> Vec<&Node>;
    fn stamp_m(&self, m: &mut DMatrix<f32>, circuit: &Circuit) {}
    fn stamp_v(&self, v: &mut DVector<f32>, circuit: &Circuit) {}
    fn is_voltage_source(&self) -> bool { false }
    fn is_inductor(&self) -> bool { false }
}

pub type Node = Option<String>;

pub const GND: &str = "GND";

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

    fn stamp_m(&self, m: &mut DMatrix<f32>, circuit: &Circuit) {
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

    fn stamp_m(&self, m: &mut DMatrix<f32>, circuit: &Circuit) {
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



#[derive(Debug)]
pub struct Circuit {
    // 回路に存在する全ての回路要素.
    pub elements: Vec<Box<dyn Element>>,

    // 回路内に存在するノードの識別子の一覧. GND を含まない.
    // この要素数が、回路の方程式の未知ベクトル V の次元となる.
    // 並び順を固定するために BTreeSet である必要がある.
    pub nodes: BTreeSet<String>,

    // 回路内に存在する電圧源の識別子の一覧.
    // この要素数が、回路の方程式の未知ベクトル I の次元となる.
    // 並び順を固定するために BTreeSet である必要がある.
    pub voltage_sources: BTreeSet<String>,
}

impl Circuit {
    pub fn new() -> Circuit {
        Circuit {
            elements: vec![],
            nodes: BTreeSet::new(),
            voltage_sources: BTreeSet::new(),
        }
    }

    // 回路素子 Element を回路に登録する
    pub fn add(&mut self, e: Box<dyn Element>) {
        // 各素子で宣言された Node を回路にも追加 ( "GND" は追加しない )
        for node in e.nodes() {
            if let Some(s) = node {
                if s != GND { self.nodes.insert(s.clone()); }
            }
        }
        // 各素子で使用された電圧源を回路にも追加.
        if e.is_voltage_source() {
            self.voltage_sources.insert(e.name().clone());
        }

        // 素子を回路の配下に移動
        self.elements.push(e);
    }

    // ノード名が回路の何番目として登録されているかを返す
    pub fn node_index(&self, my_node: &String) -> Option<usize> {
        if my_node == GND {
            return None
        }

        let mut count = 0;
        for node in &self.nodes {
            if node == my_node { break };
            count += 1;
        };
        Some(count)
    }

    // 電圧名が回路の何番目として登録されているかを返す
    pub fn voltage_index(&self, my_v_name: &String) -> usize {
        let mut count = 0;
        for v_name in &self.voltage_sources {
            if v_name == my_v_name { break };
            count += 1;
        };
        count
    }

    // 連立方程式の次元
    // 次元 = ノードの数 + 電圧/電流源の数
    pub fn circuit_eq_dim(&self) -> usize {
        self.nodes.len() +
        self.elements.iter()
          .filter(|e| e.is_voltage_source())
          .collect::<Vec<&Box<dyn Element>>>()
          .len()
    }

    // 回路の方程式左辺の行列を導出する
    pub fn circuit_eq_matrix(&self) -> DMatrix<f32> {
        let d = self.circuit_eq_dim();
        let mut a = DMatrix::<f32>::zeros(d, d);
        for e in &self.elements {
            e.stamp_m(&mut a, &self);
        }
        a
    }

    // 回路の方程式右辺のベクトルを導出する
    pub fn circuit_eq_vector(&self) -> DVector<f32> {
        let mut v = DVector::<f32>::zeros(self.circuit_eq_dim());
        for e in &self.elements {
            e.stamp_v(&mut v, &self);
        }
        v
    }

    // 回路の方程式を解く
    pub fn solve_eq(&self) -> Option<DVector<f32>> {
        let m = self.circuit_eq_matrix();
        let v = self.circuit_eq_vector();
        let m_rev = m.try_inverse();
        match m_rev {
            Some(m_rev) => Some(m_rev * v),
            _ => None
        }
    }
}

