use super::elements::*;
use downcast_rs::DowncastSync;
use nalgebra::base::{DMatrix, DVector};
use std::cell::RefCell;
use std::rc::Rc;

const SOLVER_ACCURACY: f32 = 0.0001;
const SOLVER_COUNT_MAX: u32 = 100;

// Element への内部可変性を伴った参照
// メモ: Rc<dyn Element> として Element.pins: RefCell<[Option<NodeId>; 2]> とすると
//       thread 安全性が担保できない. そのため Element 全体に内部可変性を入れる.
type ElementRef = Rc<RefCell<Box<dyn Element>>>;

// Pin への参照
#[derive(Debug)]
pub struct PinRef {
    element: ElementRef,
    pin_id: usize,
}

impl PinRef {
    pub fn new(element: &ElementRef, pin_id: usize) -> PinRef {
        PinRef {
            element: Rc::clone(element),
            pin_id: pin_id,
        }
    }
}

// シミュレーション空間の全体.
pub struct Simulator {
    // シミュレーション空間に存在する全ての回路要素を所有する
    pub elements: Vec<ElementRef>,

    // シミュレーション空間に存在する Node が結合している要素とその端子
    // 1つの Node は複数の Pin への参照を持つ.
    pub nodes: Vec<Vec<PinRef>>,
}

impl Simulator {
    pub fn new() -> Simulator {
        Simulator {
            elements: vec![],
            nodes: vec![vec![]], // node 0 is gnd
        }
    }

    pub fn add_node(&mut self) -> usize {
        self.nodes.push(vec![]);
        self.nodes.len() - 1
    }

    pub fn connect_pin_and_node(&mut self, pin: PinRef, node_id: usize) {
        // self.element と self.nodes を両方とも書き換える
        pin.element
            .borrow_mut()
            .connect_pin_to_node(pin.pin_id, node_id);
        self.nodes[node_id].push(pin);
    }

    pub fn connect_pin_and_gnd(&mut self, pin: PinRef) {
        self.connect_pin_and_node(pin, 0)
    }

    pub fn pin_ref(&self, element_id: usize, pin_id: usize) -> PinRef {
        PinRef::new(&self.elements[element_id], pin_id)
    }

    fn add_element(&mut self, element: Box<dyn Element>) -> usize {
        let element = Rc::new(RefCell::new(element));
        self.elements.push(element);
        self.elements.len() - 1
    }

    // ----------------------------------------------------------------
    // 電子部品を回路に登録するためのコード

    pub fn add_registor(&mut self, r: f32) -> usize {
        self.add_element(Box::new(Registor::new(r)))
    }

    pub fn add_diode(&mut self) -> usize {
        self.add_element(Box::new(Diode::new()))
    }

    pub fn add_ind_voltage_src(&mut self, v: f32) -> usize {
        self.add_element(Box::new(IndVoltageSrc::new(v)))
    }

    // ----------------------------------------------------------------
    // 回路の方程式を解くためのコード

    // 回路の状態ベクトルの次元.
    // 次元 = ノードの数 + 電圧/電流源の数.
    pub fn circuit_dim(&self) -> usize {
        self.nodes.len() - 1 // don't count GND node.
            + self
                .elements
                .iter()
                .filter(|elm| elm.borrow().is_voltage_src())
                .collect::<Vec<&ElementRef>>()
                .len()
    }

    pub fn create_vector(&self) -> DVector<f32> {
        DVector::<f32>::zeros(self.circuit_dim())
    }

    pub fn create_matrix(&self) -> DMatrix<f32> {
        let d = self.circuit_dim();
        DMatrix::<f32>::zeros(d, d)
    }

    // 回路の方程式の左辺行列
    // この行列は、n 回イテレーションを繰り返した回路の状態 state に依存する.
    pub fn lhs_matrix(&self, state: &DVector<f32>) -> DMatrix<f32> {
        let mut matrix = self.create_matrix();

        for elm in self
            .elements
            .iter()
            .filter(|elm| !elm.borrow().is_voltage_or_current_src())
        {
            elm.borrow().stamp_matrix(&mut matrix, state);
        }

        let offset = self.nodes.len() - 1; // don't count GND
        for (index, elm) in self
            .elements
            .iter()
            .filter(|elm| elm.borrow().is_voltage_or_current_src())
            .enumerate()
        {
            elm.borrow()
                .stamp_matrix_by_src(&mut matrix, index + offset);
        }

        matrix
    }

    // 回路の方程式の右辺ベクトル
    pub fn rhs_vector(&self, state: &DVector<f32>) -> DVector<f32> {
        let mut vector = self.create_vector();

        for elm in self
            .elements
            .iter()
            .filter(|elm| !elm.borrow().is_voltage_or_current_src())
        {
            elm.borrow().stamp_vector(&mut vector, state);
        }

        // 電圧/電流源は右辺ベクトルにスタンプを押す. そのためベクトルの index は
        //   index = 電圧以外の要素数(offset) + 電圧/電流源で何番目
        // という式で決定される.
        let offset = self.nodes.len() - 1; // don't count GND
        for (index, elm) in self
            .elements
            .iter()
            .filter(|elm| elm.borrow().is_voltage_or_current_src())
            .enumerate()
        {
            elm.borrow()
                .stamp_vector_by_src(&mut vector, index + offset);
        }
        vector
    }

    // 回路の方程式をNewton-Raphson法で解く
    pub fn solve_eq(&self) -> Option<DVector<f32>> {
        let mut vec = self.create_vector();
        let mut count = 0;
        loop {
            let lhs_matrix = self.lhs_matrix(&vec);
            let rhs_vector = self.rhs_vector(&vec);
            let l2norm = (&lhs_matrix * &vec - &rhs_vector).norm();

            // println!("|||||||||||||||||||||||||||||| count : {} |||||||||||||||||||||", count);
            // println!("||||||||||||   lhs_matrix: \n{}", &lhs_matrix);
            // println!("||||||||||||   rhs_vector: \n{}", &rhs_vector);
            // println!("||||||||||||   vec:        \n{}", &vec);
            // println!("||||||||||||   l2norm:       {}", l2norm);

            if l2norm < SOLVER_ACCURACY {
                return Some(vec);
            }
            if count > SOLVER_COUNT_MAX {
                println!("too many iter");
                return None;
            }

            match lhs_matrix.clone().try_inverse() {
                Some(matrix_rev) => {
                    // =>       M * (v - dv) = rhs_v
                    // =>            v - dv  = M^-1 * rhs_v
                    // =>   v - M^-1 * rhs_v = dv
                    // =>                 dv = v - M^-1 * rhs_v
                    let d_vec = &vec - (&matrix_rev * &rhs_vector);
                    vec = vec - d_vec;
                    count += 1;
                }
                None => {
                    println!("could not calc inverse");
                    return None;
                }
            }
        }
    }
}
