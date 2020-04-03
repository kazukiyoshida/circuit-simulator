use super::elements::element::*;
use nalgebra::base::{DMatrix, DVector};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

const SOLVER_ACCURACY: f32 = 0.0001;
const SOLVER_COUNT_MAX: u32 = 100;

pub type ElementId = usize;
pub type PinId = usize;
pub type NodeId = usize;

pub struct Simulator {
    // 回路の構成
    //   ・以下の要素をまとめたもの. この構成から一意に状態が定まる.
    //     ・回路素子の数や種類、内部パラメータ
    //     ・回路素子とノードの結合状態
    pub elements: BTreeMap<ElementId, Rc<RefCell<dyn Element>>>,
    pub nodes: BTreeSet<NodeId>,
    pub links: BTreeSet<Link>,

    // 回路の状態
    //   ・各ノードの電圧や電流.
    //   ・回路の構成が更新される度に状態は削除され、計算待ちの状態になる.
    //   ・MCU のクロックを連続で処理して変化がある場合だけ出力する、という
    //     仕組みを作るために、以前の state を内部的に保持する必要がある.
    pub state: Option<BTreeMap<NodeId, f32>>,
}

impl Simulator {
    pub fn new() -> Simulator {
        let mut nodes = BTreeSet::new();
        nodes.insert(0); // GND

        Simulator {
            elements: BTreeMap::new(),
            nodes: nodes,
            links: BTreeSet::new(),
            state: None,
        }
    }

    pub fn add_node(&mut self) -> NodeId {
        let id = self.nodes.iter().max().unwrap() + 1;
        self.nodes.insert(id);
        self.state = None;
        id
    }

    pub fn connect_element_pin_node(
        &mut self,
        element_id: ElementId,
        pin_id: PinId,
        node_id: NodeId,
    ) {
        self.elements
            .get(&element_id)
            .unwrap()
            .borrow_mut()
            .connect_pin_to_node(pin_id, node_id);
        self.links.insert(Link::new(element_id, pin_id, node_id));
        self.state = None;
    }

    pub fn connect_elment_pin_gnd(&mut self, element_id: ElementId, pin_id: PinId) {
        self.connect_element_pin_node(element_id, pin_id, 0);
    }

    // 回路の状態
    fn state(&mut self) -> Result<BTreeMap<NodeId, f32>, String> {
        match self.solve_eq() {
            Ok(eq) => {
                let mut state = BTreeMap::new();
                for node_id in self.nodes.iter() {
                    // except GND
                    if *node_id != 0 {
                        let index = eq.node_index.get(node_id).unwrap();
                        state.insert(*node_id, eq.x[*index]);
                    }
                }
                Ok(state)
            }
            Err(err) => {
                // 失敗した場合は電源を落とした状態にしてみる..
                let mut state = BTreeMap::new();
                for node_id in self.nodes.iter() {
                    state.insert(*node_id, 0.0);
                }
                Ok(state)
            }
        }
    }

    // 回路の状態を求める（定常状態を計算する）
    pub fn update_state(&mut self) -> Result<BTreeMap<NodeId, f32>, String> {
        match self.state() {
            Ok(state) => {
                self.state = Some(state.clone());
                Ok(state)
            }
            Err(err) => Err(err),
        }
    }

    // 回路の状態を求める（非定常状態を計算する）
    //   MCU のクロックを進め、以前の状態から変化がある場合にだけ更新後の状態を返す.
    pub fn next(&mut self) -> Result<Option<BTreeMap<NodeId, f32>>, String> {
        // 状態があらかじめ計算されていないと変化を検出できない.
        if self.state.is_none() {
            return Err("no state calculated".to_string());
        }

        // MCU のクロックを進める
        let is_updated = self
            .elements
            .values()
            .fold(false, |sum, element| sum || element.borrow().clk());

        // クロックを進めたものの回路の構成に変化がない（ IOPort などに変化がない）場合は終了
        if !is_updated {
            return Ok(None);
        }

        // 回路の構成が変わったので状態を再計算. 変化がある場合だけ値を返す.
        match self.state() {
            Ok(state) => {
                if self.state.as_ref() == Some(&state) {
                    Ok(None)
                } else {
                    self.state = Some(state.clone());
                    Ok(Some(state))
                }
            }
            Err(err) => Err(err),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Link {
    element_id: ElementId,
    pin_id: PinId,
    node_id: NodeId,
}

impl Link {
    fn new(element_id: ElementId, pin_id: PinId, node_id: NodeId) -> Link {
        Link {
            element_id: element_id,
            pin_id: pin_id,
            node_id: node_id,
        }
    }
}

// Ax = z
#[derive(Debug)]
pub struct Equation {
    pub a: DMatrix<f32>,
    pub x: DVector<f32>,
    pub z: DVector<f32>,

    // node_id が方程式の何段目に当たるか.
    pub node_index: BTreeMap<NodeId, usize>,
    // src となっている Element の element_id が方程式の何段目に当たるか.
    pub src_index: BTreeMap<(ElementId, PinId), usize>,
}

#[derive(Debug)]
pub enum EqSolveError {
    RevMatrix,
    MaxIteration,
}

impl Simulator {
    // 回路の状態ベクトルの次元.
    // 次元 = ノードの数 + 電圧/電流源となっているピンの数.
    fn equation_dim(&self) -> usize {
        self.nodes.len() - 1 // don't count GND node.
            + self
                .elements
                .values()
                .fold(0, |mut sum, element| {
                    sum += element
                             .borrow()
                             .output_pins()
                             .iter()
                             .filter(|&is_output| *is_output)
                             .collect::<Vec<&bool>>()
                             .len();
                    sum
                })
    }

    fn create_equation(&self) -> Equation {
        let dim = self.equation_dim();
        let mut eq = Equation {
            a: DMatrix::<f32>::zeros(dim, dim),
            x: DVector::<f32>::zeros(dim),
            z: DVector::<f32>::zeros(dim),
            node_index: BTreeMap::new(),
            src_index: BTreeMap::new(),
        };

        for (index, node_id) in self.nodes.iter().enumerate() {
            // don't count GND
            if node_id > &0 {
                eq.node_index.insert(*node_id, index - 1);
            }
        }

        let mut index = 0;
        for (element_id, element) in self.elements.iter() {
            for (pin_id, is_output_pin) in element.borrow().output_pins().iter().enumerate() {
                if *is_output_pin {
                    eq.src_index.insert((*element_id, pin_id), index);
                    index += 1;
                }
            }
        }

        eq
    }

    // 方程式の左辺行列 A と右辺ベクトル z にスタンプを押す
    fn stamp_equation(&self, mut eq: &mut Equation) {
        // スタンプを押す前は A, z は初期化する.
        let dim = self.equation_dim();
        eq.a = DMatrix::<f32>::zeros(dim, dim);
        eq.z = DVector::<f32>::zeros(dim);

        for element in self.elements.values() {
            element.borrow().stamp(&mut eq);
        }
    }

    // 方程式を Newton-Raphson 法で解く
    fn solve_eq(&mut self) -> Result<Equation, EqSolveError> {
        let mut eq = self.create_equation();
        for _ in 0..SOLVER_COUNT_MAX {
            self.stamp_equation(&mut eq);

            let l2norm = (&eq.a * &eq.x - &eq.z).norm();
            // println!("||| eq.a: {}", eq.a);
            // println!("||| eq.x: {}", eq.x);
            // println!("||| eq.z: {}", eq.z);
            // println!("||| eq.node_index: {:?}", eq.node_index);
            // println!("||| eq.src_index: {:?}", eq.src_index);
            // println!("l2:  {}", l2norm);
            if l2norm < SOLVER_ACCURACY {
                return Ok(eq);
            }

            match eq.a.clone().try_inverse() {
                Some(a_rev) => {
                    // A * (x - dx) = z  =>  dx = x - A^-1 * z
                    let dx = &eq.x - (&a_rev * &eq.z);
                    eq.x = &eq.x - &dx;
                }
                None => return Err(EqSolveError::RevMatrix),
            }
        }
        return Err(EqSolveError::MaxIteration);
    }
}
