use super::circuit::*;
use downcast_rs::DowncastSync;
use nalgebra::base::{DMatrix, DVector};

pub trait Element: std::fmt::Debug + DowncastSync {
    fn name(&self) -> &String;
    fn nodes(&self) -> Vec<&Node>;

    fn stamp_m(&self, _matrix: &mut DMatrix<f32>, _circuit: &Circuit, _state: &DVector<f32>) {}

    fn stamp_v(&self, _vector: &mut DVector<f32>, _circuit: &Circuit) {}

    fn is_voltage_source(&self) -> bool {
        false
    }
}

impl_downcast!(sync Element);

// ============================================================================
// Registor

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

    fn g(&self) -> f32 {
        1_f32 / self.r
    }
}

impl Element for Resistor {
    fn name(&self) -> &String {
        &self.name
    }

    fn nodes(&self) -> Vec<&Node> {
        vec![&self.nodes[0], &self.nodes[1]]
    }

    fn stamp_m(&self, m: &mut DMatrix<f32>, circuit: &Circuit, _: &DVector<f32>) {
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

// ============================================================================
// Diode

#[derive(Debug)]
pub struct Diode {
    pub name: String,
    pub nodes: [Node; 2],

    // モデル用パラメータ
    pub v_thr: f32,
    pub g_d: f32,
}

impl Diode {
    pub fn new(name: String, nodes: [Node; 2]) -> Diode {
        Diode {
            name: name,
            nodes: nodes,
            v_thr: 0.674_f32,
            g_d: 0.191_f32,
        }
    }

    // 順方向電圧 Vd における電流 I(Vd) を以下の式でモデリングする.
    // ※ 指数関数でモデリングするとNewton法で値が発散するため区分線形近似する.
    //
    // I(Vd) = 0             ( Vd <= Vthr )
    //       = Gd(Vd - Vthr) ( Vd > Vthr )
    //
    pub fn i(&self, vd: f32) -> f32 {
        if vd <= self.v_thr {
            0f32
        } else {
            self.g_d * (vd - self.v_thr)
        }
    }

    pub fn di_dv(&self, vd: f32) -> f32 {
        if vd <= self.v_thr {
            0f32
        } else {
            self.g_d
        }
    }
}

impl Element for Diode {
    fn name(&self) -> &String {
        &self.name
    }

    fn nodes(&self) -> Vec<&Node> {
        vec![&self.nodes[0], &self.nodes[1]]
    }

    fn stamp_m(&self, m: &mut DMatrix<f32>, circuit: &Circuit, state: &DVector<f32>) {
        // 自分のノード
        let n0 = self.nodes[0].as_ref().unwrap();
        let n1 = self.nodes[1].as_ref().unwrap();

        // 回路のノードを参照して行列のインデックスを決定
        let i0 = circuit.node_index(n0);
        let i1 = circuit.node_index(n1);

        let didv = self.di_dv(state[0]);

        if let (Some(i0_), Some(i1_)) = (i0, i1) {
            m[(i0_, i0_)] += didv;
            m[(i1_, i1_)] += didv;
            m[(i0_, i1_)] -= didv;
            m[(i1_, i0_)] -= didv;
        } else {
            if let Some(i) = i0 {
                m[(i, i)] += didv;
            }
            if let Some(i) = i1 {
                m[(i, i)] += didv;
            }
        }
    }
}

// ============================================================================
// IndependentVoltageSource

#[derive(Debug)]
pub struct IndVoltageSrc {
    pub name: String,
    pub v: f32,
    pub nodes: [Node; 2],
}

impl Element for IndVoltageSrc {
    fn name(&self) -> &String {
        &self.name
    }

    fn nodes(&self) -> Vec<&Node> {
        vec![&self.nodes[0], &self.nodes[1]]
    }

    fn stamp_m(&self, m: &mut DMatrix<f32>, circuit: &Circuit, _: &DVector<f32>) {
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

    fn is_voltage_source(&self) -> bool {
        true
    }
}

// ============================================================================
// IndependentCurrentSource

#[derive(Debug)]
pub struct IndCurrentSrc {
    pub name: String,
    pub i: f32,
    pub nodes: [Node; 2],
}

impl Element for IndCurrentSrc {
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
