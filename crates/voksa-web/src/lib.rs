//! wasm-bindgen surface for voksa. Built with:
//! `wasm-pack build --release --target web crates/voksa-web`

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn version() -> String {
    voksa_core::VERSION.into()
}

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    voksa_core::greet(name)
}

#[cfg(test)]
mod tests {
    #[test]
    fn greet_delegates_to_core() {
        assert_eq!(super::greet("munje"), "coi munje");
    }
}
