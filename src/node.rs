use std::cell::RefCell;
use std::collections::HashSet;
use wasm_bindgen::prelude::*;

thread_local! {
    pub static NODES: RefCell<Nodes> = RefCell::new(Nodes::new());
}

// TODO: static NODES が wasm で公開できるなら、これは不要
#[wasm_bindgen]
pub fn add_nodes() -> usize {
    NODES.with(|nodes| nodes.borrow_mut().add_nodes())
}

#[wasm_bindgen]
pub struct Nodes {
    set: HashSet<usize>,
    count: usize,
}

impl Nodes {
    fn new() -> Nodes {
        let mut set = HashSet::new();
        set.insert(0); // GND
        Nodes { set: set, count: 1 }
    }

    pub fn add_nodes(&mut self) -> usize {
        let node_id = self.count;
        self.set.insert(node_id);
        self.count += 1;
        node_id
    }

    pub fn len(&self) -> usize {
        self.set.len()
    }
}
