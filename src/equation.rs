use super::node::*;
use nalgebra::base::{DMatrix, DVector};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

const SOLVER_ACCURACY: f32 = 0.0001;
const SOLVER_COUNT_MAX: u32 = 100;

thread_local! {
    static EQUATION: RefCell<Equation> = RefCell::new(Equation::new());
}

// Ax = z
struct Equation {
    a: DMatrix<f32>,
    x: DVector<f32>,
    z: DVector<f32>,
}

impl Equation {
    fn new() -> Equation {
        Equation {
            a: DMatrix::<f32>::zeros(0, 0),
            x: DVector::<f32>::zeros(0),
            z: DVector::<f32>::zeros(0),
        }
    }
}

// 1. 回路の方程式を初期化する
#[wasm_bindgen]
pub fn initialize(src_count: usize) {
    // 方程式の次元 = ノードの数 + 電圧/電流源の数.
    let dim = NODES.with(|nodes| nodes.borrow().len() - 1 + src_count);
    EQUATION.with(|eq| {
        eq.borrow_mut().a = DMatrix::<f32>::zeros(dim, dim);
        eq.borrow_mut().x = DVector::<f32>::zeros(dim);
        eq.borrow_mut().z = DVector::<f32>::zeros(dim);
    })
}

// 2. 全ての Element でスタンプを押す.

// 3. 回路の方程式を Newton 法で解く
#[wasm_bindgen]
pub fn solve() {
    // EQUATION.with(|eq| {
    //     let mut count = 0;
    //     let a = &eq.borrow().a;
    //     let x = &eq.borrow().x;
    //     let z = &eq.borrow().z;
    //     for _ in 0..SOLVER_COUNT_MAX {
    //         let l2 = (a * x - z).norm();
    //         if l2 < SOLVER_ACCURACY {
    //             return Some(x);
    //         }

    //         // A * (x - dx) = z   =>   dx = x - A^-1 * z
    //         let a_rev = a.clone().try_inverse().unwrap();
    //         let dx = x - (a_rev * x);
    //         eq.borrow_mut().x = x - &dx;
    //     }
    //     println!("too many iter");
    //     None
    // });
    // // TODO: return を決める
}
