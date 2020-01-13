extern crate ndarray;

use ndarray::prelude::*;


pub trait Element: std::fmt::Debug {
    fn nodes(&self) -> Vec<&Node>;
}

pub type Node = Option<String>;

// 抵抗（抵抗値 (Ω)）
#[derive(Debug)]
pub struct Resistor {
    pub name: String,
    pub r: u16,
    pub nodes: [Node; 2],
}

impl Element for Resistor {
    fn nodes(&self) -> Vec<&Node> {
        vec![&self.nodes[0], &self.nodes[1]]
    }
}

// // キャパシタ（容量値 (F)）
// #[derive(Debug)]
// pub struct Capacitor {
//     pub name: String,
//     pub f: u16,
// }
//
// impl Element for Capacitor {
//     fn nodes(&self) -> Vec<&Node> {
//         vec![]
//     }
// }
//
// // ダイオード
// #[derive(Debug)]
// pub struct Diode {
//     pub name: String,
// }
//
// impl Element for Diode {
//     fn nodes(&self) -> Vec<&Node> {
//         vec![]
//     }
// }
//
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
//
// // 独立電圧源（電圧値(V)）
// #[derive(Debug)]
// pub struct VoltageSource {
//     pub name: String,
//     pub v: u16,
// }
//
// impl Element for VoltageSource {
//     fn nodes(&self) -> Vec<&Node> {
//         vec![]
//     }
// }


#[derive(Debug)]
pub struct Circuit {
    pub elements: Vec<Box<dyn Element>>,
    pub nodes: Vec<String>,
}

impl Circuit {
    pub fn new() -> Circuit {
        Circuit {
            elements: vec![],
            nodes: vec![],
        }
    }

    // 回路素子 Element を回路に登録する
    pub fn add(&mut self, e: Box<dyn Element>) {
        // 各素子で宣言された Node を回路にも登録
        for node in e.nodes() {
            match node {
                Some(s) => self.nodes.push(s.clone()),
                _       => (),
            }
        }
        // 素子を回路の配下に移動
        self.elements.push(e);
    }
}

// 回路から方程式を導出する
pub fn create_matrix(c: &Circuit) -> Array2<i16> {
    Array2::eye(3)
}

