use downcast_rs::DowncastSync;
use nalgebra::base::{DMatrix, DVector};
use std::cell::RefCell;
use std::rc::Rc;

const SOLVER_ACCURACY: f32 = 0.0001;
const SOLVER_COUNT_MAX: u32 = 100;

// 各要素の参照関係
// Element 1 : N Pin M : 1 Node

// Node への参照
type NodeId = usize;

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

pub trait Element: std::fmt::Debug + DowncastSync {
    // Element の pin を Node＿に繋げる
    fn connect_pin_to_node(&mut self, pin_id: usize, node_id: usize);

    // その要素が電圧源であるか
    fn is_voltage_src(&self) -> bool {
        false
    }
    // その要素が電流源であるか
    fn is_current_src(&self) -> bool {
        false
    }
    // その要素が電流/電圧源であるか
    fn is_voltage_or_current_src(&self) -> bool {
        self.is_voltage_src() || self.is_current_src()
    }

    // 修正節点法における回路の方程式の行列へのスタンプ
    fn stamp_matrix(&self, _: &mut DMatrix<f32>, _: &DVector<f32>) {}
    fn stamp_matrix_by_src(&self, _: &mut DMatrix<f32>, _: usize) {}

    // 修正節点法における回路の方程式のベクトルへのスタンプ
    fn stamp_vector(&self, _: &mut DVector<f32>, _: &DVector<f32>) {}
    fn stamp_vector_by_src(&self, _: &mut DVector<f32>, _: usize) {}
}

#[derive(Debug)]
pub struct Registor {
    pins: [Option<NodeId>; 2],
    resistance: f32,
}

impl Registor {
    pub fn new(registance: f32) -> Registor {
        let r = if registance == 0.0 { 0.01 } else { registance };
        Registor {
            pins: [None, None],
            resistance: r,
        }
    }

    pub fn conductance(&self) -> f32 {
        1.0 / self.resistance
    }
}

impl Element for Registor {
    fn connect_pin_to_node(&mut self, pin_id: usize, node_id: usize) {
        self.pins[pin_id] = Some(node_id);
    }

    fn stamp_matrix(&self, matrix: &mut DMatrix<f32>, _: &DVector<f32>) {
        // shift 1 for GND ...
        let p0 = self.pins[0].and_then(|n| n.checked_sub(1usize));
        let p1 = self.pins[1].and_then(|n| n.checked_sub(1usize));
        match [p0, p1] {
            [Some(p0), Some(p1)] => {
                matrix[(p0, p0)] += self.conductance();
                matrix[(p1, p1)] += self.conductance();
                matrix[(p0, p1)] -= self.conductance();
                matrix[(p1, p0)] -= self.conductance();
            }
            [Some(p0), None] => matrix[(p0, p0)] += self.conductance(),
            [None, Some(p1)] => matrix[(p1, p1)] += self.conductance(),
            [None, None] => {}
        };
    }
}

// ダイオード
// 順方向電圧 Vd における電流 I(Vd) を区分線形近似でモデリングする.
// I(V) = 0                      ( Vd <= threshold )
//      = grad * (V - threshold) ( Vd  > threshold )
#[derive(Debug)]
pub struct Diode {
    // pins[0] : Anode
    // pins[1] : Cathode
    pins: [Option<NodeId>; 2],
    threshold: f32,
    grad: f32,
}

impl Diode {
    pub fn new() -> Diode {
        Diode {
            pins: [None, None],
            threshold: 0.674,
            grad: 0.191,
        }
    }

    pub fn current(&self, volt: f32) -> f32 {
        if volt <= self.threshold {
            0.0
        } else {
            self.grad * (volt - self.threshold)
        }
    }

    pub fn d_current(&self, volt: f32) -> f32 {
        if volt <= self.threshold {
            0.0
        } else {
            self.grad
        }
    }
}

impl Element for Diode {
    fn connect_pin_to_node(&mut self, pin_id: usize, node_id: usize) {
        self.pins[pin_id] = Some(node_id);
    }

    // cf. https://spicesharp.github.io/SpiceSharp/articles/custom_components/modified_nodal_analysis.html
    fn stamp_vector(&self, rhs_vec: &mut DVector<f32>, state: &DVector<f32>) {
        // shift 1 for GND ...
        let p0 = self.pins[0].and_then(|n| n.checked_sub(1usize)); // anode
        let p1 = self.pins[1].and_then(|n| n.checked_sub(1usize)); // cathode

        match [p0, p1] {
            [Some(p0), Some(p1)] => {
                // let didv = self.d_current(state[p0] - state[p1]);
                // println!("##### state[p0]  : {}", state[p0]);
                // println!("##### state[p1]  : {}", state[p1]);
                // println!("##### d_current  : {}", didv);
                // matrix[(p0, p0)] += didv;
                // matrix[(p1, p1)] += didv;
                // matrix[(p0, p1)] -= didv;
                // matrix[(p1, p0)] -= didv;
            }
            [Some(p0), None] => {
                if state[p0] >= self.threshold {
                    rhs_vec[p0] += self.threshold * self.grad;
                }
            }
            [None, _] => {}
        };
    }

    fn stamp_matrix(&self, matrix: &mut DMatrix<f32>, state: &DVector<f32>) {
        // shift 1 for GND ...
        let p0 = self.pins[0].and_then(|n| n.checked_sub(1usize)); // anode
        let p1 = self.pins[1].and_then(|n| n.checked_sub(1usize)); // cathode
        match [p0, p1] {
            [Some(p0), Some(p1)] => {
                let didv = self.d_current(state[p0] - state[p1]);
                matrix[(p0, p0)] += didv;
                matrix[(p1, p1)] += didv;
                matrix[(p0, p1)] -= didv;
                matrix[(p1, p0)] -= didv;
            }
            [Some(p0), None] => {
                let didv = self.d_current(state[p0]);
                matrix[(p0, p0)] += didv;
            }
            [None, _] => {}
        };
    }
}

#[derive(Debug)]
pub struct IndVoltageSrc {
    pins: [Option<NodeId>; 2],
    voltage: f32,
}

impl IndVoltageSrc {
    pub fn new(volt: f32) -> IndVoltageSrc {
        IndVoltageSrc {
            pins: [None, None],
            voltage: volt,
        }
    }
}

impl Element for IndVoltageSrc {
    fn connect_pin_to_node(&mut self, pin_id: usize, node_id: usize) {
        self.pins[pin_id] = Some(node_id);
    }

    fn is_voltage_src(&self) -> bool {
        true
    }

    fn stamp_vector_by_src(&self, vector: &mut DVector<f32>, index: usize) {
        vector[index] = self.voltage;
    }

    fn stamp_matrix_by_src(&self, matrix: &mut DMatrix<f32>, index: usize) {
        // shift 1 for GND ...
        let p0 = self.pins[0].and_then(|n| n.checked_sub(1usize));
        let p1 = self.pins[1].and_then(|n| n.checked_sub(1usize));
        match [p0, p1] {
            [Some(p0), Some(p1)] => {
                matrix[(p0, index)] = 1.0;
                matrix[(p1, index)] = -1.0;
                matrix[(index, p0)] = 1.0;
                matrix[(index, p1)] = -1.0;
            }
            [Some(p0), None] => {
                matrix[(p0, index)] = 1.0;
                matrix[(index, p0)] = 1.0;
            }
            [None, Some(p1)] => {
                matrix[(p1, index)] = 1.0;
                matrix[(index, p1)] = 1.0;
            }
            [None, None] => {}
        };
    }
}
