use std::cell::RefCell;
use std::collections::HashSet;

thread_local! {
    pub static LINKS: RefCell<Links> = RefCell::new(Links::new());
}

pub struct Links(HashSet<Link>);

impl Links {
    pub fn new() -> Links {
        Links(HashSet::new())
    }

    pub fn add(&mut self, element_id: usize, pin_id: usize, node_id: usize) {
        let link = Link::new(element_id, pin_id, node_id);
        self.0.insert(link);
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Link {
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
