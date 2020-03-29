use super::elements::*;
use nalgebra::base::{DMatrix, DVector};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

const SOLVER_ACCURACY: f32 = 0.0001;
const SOLVER_COUNT_MAX: u32 = 100;

#[derive(Debug, PartialEq, Eq, Hash)]
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
    pub elements: HashMap<usize, Rc<RefCell<dyn Element>>>,
    pub nodes: HashSet<usize>,
    pub links: HashSet<Link>,
    pub eq: Equation,
}

// #[wasm_bindgen]
impl Simulator {
    // #[wasm_bindgen(constructor)]
    pub fn new() -> Simulator {
        let mut nodes = HashSet::new();
        nodes.insert(0); // GND

        Simulator {
            elements: HashMap::new(),
            nodes: nodes,
            links: HashSet::new(),
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
    // 電子部品を回路に登録するためのコード

    pub fn add_registor(&mut self, r: f32) -> usize {
        let id = self.elements.keys().max().unwrap_or(&0usize) + 1;
        let element = Rc::new(RefCell::new(Registor::new(id, r)));
        self.elements.insert(id, element);
        id
    }

    pub fn add_diode(&mut self) -> usize {
        let id = self.elements.keys().max().unwrap_or(&0usize) + 1;
        let element = Rc::new(RefCell::new(Diode::new(id)));
        self.elements.insert(id, element);
        id
    }

    pub fn add_ind_voltage_src(&mut self, v: f32) -> usize {
        let id = self.elements.keys().max().unwrap_or(&0usize) + 1;
        let element = Rc::new(RefCell::new(IndVoltageSrc::new(id, v)));
        self.elements.insert(id, element);
        id
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
            println!("|||||  OK !! ||||");
            self.eq.node_index.insert(*node_id, index);
        }

        for (index, (element_id, element)) in self
            .elements
            .iter()
            .filter(|&(id, e)| e.borrow().is_voltage_or_current_src())
            .enumerate()
        {
            println!("|||||  OK !! ||||");
            self.eq.src_index.insert(*element_id, index);
        }
    }

    // 方程式を解くイテレーションの度にスタンプを押す
    fn stamp_equation(&mut self) {
        for element in self.elements.values() {
            element.borrow().stamp(&mut self.eq);
        }
    }

    // 方程式をNewton-Raphson法で解く
    pub fn solve_eq(&mut self) -> Option<&DVector<f32>> {
        self.create_equation();

        println!("||| elements: {:?}", self.elements);
        println!("||| nodes: {:?}", self.nodes);
        println!("||| links: {:?}", self.links);

        for _ in 0..SOLVER_COUNT_MAX {
            self.stamp_equation();

            println!("||| eq.a: {}", self.eq.a);
            println!("||| eq.x: {}", self.eq.x);
            println!("||| eq.z: {}", self.eq.z);
            println!("||| eq.node_index: {:?}", self.eq.node_index);
            println!("||| eq.src_index: {:?}", self.eq.src_index);

            let l2norm = (&self.eq.a * &self.eq.x - &self.eq.z).norm();

            // println!("||||| count : {} ||||", count);
            // println!("A: \n{}", &self.eq.a);
            // println!("x: \n{}", &self.eq.x);
            // println!("z: \n{}", &self.eq.z);
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
    pub node_index: HashMap<usize, usize>,
    // src となっている Element の element_id が方程式の何段目に当たるか. element_id: index
    pub src_index: HashMap<usize, usize>,
}

impl Equation {
    fn new() -> Equation {
        Equation {
            a: DMatrix::<f32>::zeros(0, 0),
            x: DVector::<f32>::zeros(0),
            z: DVector::<f32>::zeros(0),
            node_index: HashMap::new(),
            src_index: HashMap::new(),
        }
    }
}
