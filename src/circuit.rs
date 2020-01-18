extern crate nalgebra;

use std::collections::BTreeSet;
use nalgebra::base::{DMatrix, DVector};
use super::elements::*;

pub type Node = Option<String>;

pub const GND: &str = "GND";

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

    // 回路の状態ベクトルの次元.
    // 連立方程式の次元に当たる. 次元 = ノードの数 + 電圧/電流源の数.
    pub fn circuit_eq_dim(&self) -> usize {
        self.nodes.len() +
        self.elements.iter()
          .filter(|e| e.is_voltage_source())
          .collect::<Vec<&Box<dyn Element>>>()
          .len()
    }

    // 回路の方程式を解いて状態を決定する
    pub fn solve_eq(&self) -> Option<DVector<f32>> {
        let solver = CircuitSolver::new(&self, 0.01);
        solver.solve()
    }
}


#[derive(Debug)]
pub struct CircuitSolver<'t> {
    // 解析対象となる回路.
    pub circuit: &'t Circuit,
    // 回路の方程式の n 回計算したタイミングでの状態.
    pub state: (DVector<f32>, u32),
    // 解の精度.
    pub accuracy: f32,
}

impl<'t> CircuitSolver<'t> {
    pub fn new(circuit: &'t Circuit, a: f32) -> CircuitSolver<'t> {
        let v = DVector::<f32>::zeros(circuit.circuit_eq_dim());
        CircuitSolver {
            circuit: circuit,
            state: (v, 0_u32),
            accuracy: a,
        }
    }

    // 回路の方程式左辺の行列を導出する
    // この行列は、n をイテレーション数として、回路の状態Vn に依存する.
    pub fn matrix(&self) -> DMatrix<f32> {
        let d = self.circuit.circuit_eq_dim();
        let mut a = DMatrix::<f32>::zeros(d, d);
        for e in &self.circuit.elements {
            let (s, _) = &self.state;
            e.stamp_m(&mut a, &self.circuit, s);
        }
        a
    }

    // 回路の方程式右辺のベクトルを導出する
    // このベクトルは、n をイテレーション数として、回路の状態Vn に依存する.
    pub fn vector(&self) -> DVector<f32> {
        let mut v = DVector::<f32>::zeros(self.circuit.circuit_eq_dim());
        for e in &self.circuit.elements {
            e.stamp_v(&mut v, &self.circuit);
        }
        v
    }

    // 回路の方程式を解く
    pub fn solve(&self) -> Option<DVector<f32>> {
        let m = self.matrix();
        let v = self.vector();
        let m_rev = m.try_inverse();
        match m_rev {
            Some(m_rev) => Some(m_rev * v),
            _ => None
        }
    }
}

