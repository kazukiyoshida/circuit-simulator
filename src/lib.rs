extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

#[macro_use]
extern crate downcast_rs;

pub mod circuit;
pub mod elements;

// extern は「外で定義されているxxxを取り込む」という意味.
// extern crate も同様.
#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}
