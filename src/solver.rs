use super::circuit::*;
use nalgebra::base::{DMatrix, DVector};

#[derive(Debug)]
pub struct CircuitSolver<'t> {
    // 解析対象となる回路.
    pub circuit: &'t Circuit,
    // 解の精度.
    pub accuracy: f32,
    // Newton-Raphson法の最大イテレーション数
    pub max_iter: u32,
}

impl<'t> CircuitSolver<'t> {
    pub fn new(circuit: &'t Circuit, a: f32) -> CircuitSolver<'t> {
        CircuitSolver {
            circuit: circuit,
            accuracy: a,
            max_iter: 100u32,
        }
    }

    // 回路の方程式左辺の行列を導出する
    // この行列は、n をイテレーション数として、回路の状態Vn に依存する.
    pub fn matrix(&self, state: &DVector<f32>) -> DMatrix<f32> {
        let d = self.circuit.circuit_eq_dim();
        let mut a = DMatrix::<f32>::zeros(d, d);
        for e in &self.circuit.elements {
            e.stamp_m(&mut a, &self.circuit, state);
        }
        a
    }

    // 回路の方程式右辺のベクトルを導出する
    // このベクトルは、n をイテレーション数として、回路の状態Vn に依存する.
    // WIP: stamp_v を状態 state に依存させる
    pub fn rhs_vector(&self, _: &DVector<f32>) -> DVector<f32> {
        let mut v = DVector::<f32>::zeros(self.circuit.circuit_eq_dim());
        for e in &self.circuit.elements {
            e.stamp_v(&mut v, &self.circuit);
        }
        v
    }

    // 回路の方程式をNewton-Raphson法で解く
    pub fn solve(&self) -> Option<DVector<f32>> {
        let mut v = DVector::<f32>::zeros(self.circuit.circuit_eq_dim());
        let mut iter = 0u32;

        loop {
            // 状態 v_iter における行列・右辺ベクトル、およびその時の L2 ノルムを計算.
            let jacobian = self.matrix(&v);
            let rhs = self.rhs_vector(&v);
            let l2 = (&jacobian * &v - &rhs).norm();

            // println!("|||||||||||| iter : {} |||||||||||||||||||||", iter);
            // println!("jacobian: \n{}", &jacobian);
            // println!("rhs:      \n{}", &rhs);
            // println!("v:        \n{}", &v);
            // println!("l2:         {}", l2);

            if l2 < self.accuracy {
                break Some(v);
            }
            if iter > self.max_iter {
                println!(" #### calc error, too many iteration. ####");
                break None;
            }

            // 次の状態 v_(iter+1) を計算
            let jacobian_rev = jacobian.clone().try_inverse();
            match jacobian_rev {
                Some(jacobian_rev) => {
                    let dv = &jacobian_rev * (&jacobian * &v - &rhs);
                    v = v - dv;
                    iter += 1;
                }
                None => break None,
            }
        }
    }
}
