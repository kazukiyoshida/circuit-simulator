use super::elements::element::*;
use nalgebra::base::{DMatrix, DVector};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

const SOLVER_ACCURACY: f32 = 0.0001;
const SOLVER_COUNT_MAX: u32 = 100;

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Link {
    element_id: usize,
    pin_id: usize,
    node_id: usize,
}

impl Link {
    fn new(element_id: usize, pin_id: usize, node_id: usize) -> Link {
        Link {
            element_id: element_id,
            pin_id: pin_id,
            node_id: node_id,
        }
    }
}

// シミュレーション空間の全体.
// #[wasm_bindgen]
pub struct Simulator {
    pub elements: BTreeMap<usize, Rc<RefCell<dyn Element>>>,
    pub nodes: BTreeSet<usize>,
    pub links: BTreeSet<Link>,
    pub eq: Equation,
}

// #[wasm_bindgen]
impl Simulator {
    // #[wasm_bindgen(constructor)]
    pub fn new() -> Simulator {
        let mut nodes = BTreeSet::new();
        nodes.insert(0); // GND

        Simulator {
            elements: BTreeMap::new(),
            nodes: nodes,
            links: BTreeSet::new(),
            eq: Equation::new(),
        }
    }

    pub fn add_node(&mut self) -> usize {
        let id = self.nodes.iter().max().unwrap() + 1;
        self.nodes.insert(id);
        id
    }

    pub fn connect_element_pin_node(&mut self, element_id: usize, pin_id: usize, node_id: usize) {
        self.elements
            .get(&element_id)
            .unwrap()
            .borrow_mut()
            .connect_pin_to_node(pin_id, node_id);
        self.links.insert(Link::new(element_id, pin_id, node_id));
    }

    pub fn connect_elment_pin_gnd(&mut self, element_id: usize, pin_id: usize) {
        self.connect_element_pin_node(element_id, pin_id, 0);
    }

    // ----------------------------------------------------------------
    // 回路の方程式を解くためのコード

    // 回路の状態ベクトルの次元.
    // 次元 = ノードの数 + 電圧/電流源の数.
    fn equation_dim(&self) -> usize {
        self.nodes.len() - 1 // don't count GND node.
            + self
                .elements
                .values()
                .fold(0, |mut sum, e| {
                    if e.borrow().is_voltage_or_current_src() {
                        sum += 1;
                    }
                    sum
                })
    }

    // Node の追加・削除、Element の追加・削除の度に方程式も初期化する
    fn create_equation(&mut self) {
        let dim = self.equation_dim();
        self.eq.a = DMatrix::<f32>::zeros(dim, dim);
        self.eq.x = DVector::<f32>::zeros(dim);
        self.eq.z = DVector::<f32>::zeros(dim);

        for (index, node_id) in self.nodes.iter().enumerate() {
            // don't count GND
            if node_id > &0 {
                self.eq.node_index.insert(*node_id, index - 1);
            }
        }

        for (index, (element_id, _)) in self
            .elements
            .iter()
            .filter(|&(_, e)| e.borrow().is_voltage_or_current_src())
            .enumerate()
        {
            self.eq.src_index.insert(*element_id, index);
        }
    }

    // 方程式を解くイテレーションの度にスタンプを押す
    fn stamp_equation(&mut self) {
        let dim = self.equation_dim();
        self.eq.a = DMatrix::<f32>::zeros(dim, dim);
        self.eq.z = DVector::<f32>::zeros(dim);
        for element in self.elements.values() {
            element.borrow().stamp(&mut self.eq);
        }
    }

    // 方程式をNewton-Raphson法で解く
    pub fn solve_eq(&mut self) -> Option<&DVector<f32>> {
        self.create_equation();
        for _ in 0..SOLVER_COUNT_MAX {
            self.stamp_equation();

            let l2norm = (&self.eq.a * &self.eq.x - &self.eq.z).norm();

            // println!("||||| count : {} ||||", count);
            // println!("||| eq.a: {}", self.eq.a);
            // println!("||| eq.x: {}", self.eq.x);
            // println!("||| eq.z: {}", self.eq.z);
            // println!("||| eq.node_index: {:?}", self.eq.node_index);
            // println!("||| eq.src_index: {:?}", self.eq.src_index);
            // println!("l2:  {}", l2);
            if l2norm < SOLVER_ACCURACY {
                return Some(&self.eq.x);
            }

            match self.eq.a.clone().try_inverse() {
                Some(a_rev) => {
                    // A * (x - dx) = z  =>  dx = x - A^-1 * z
                    let dx = &self.eq.x - (&a_rev * &self.eq.z);
                    self.eq.x = &self.eq.x - &dx;
                }
                None => {
                    println!("could not calc inverse");
                    return None;
                }
            }
        }
        println!("too many iteration");
        return None;
    }
}

// Ax = z
#[derive(Debug)]
pub struct Equation {
    pub a: DMatrix<f32>,
    pub x: DVector<f32>,
    pub z: DVector<f32>,

    // node_id が方程式の何段目に当たるか. node_id: index
    pub node_index: BTreeMap<usize, usize>,
    // src となっている Element の element_id が方程式の何段目に当たるか. element_id: index
    pub src_index: BTreeMap<usize, usize>,
}

impl Equation {
    fn new() -> Equation {
        Equation {
            a: DMatrix::<f32>::zeros(0, 0),
            x: DVector::<f32>::zeros(0),
            z: DVector::<f32>::zeros(0),
            node_index: BTreeMap::new(),
            src_index: BTreeMap::new(),
        }
    }
}
