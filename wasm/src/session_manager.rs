// Browser-side cloak session lifecycle — Phase 3
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct CloakSession {
    address: String,
}

#[wasm_bindgen]
impl CloakSession {
    #[wasm_bindgen(constructor)]
    pub fn new(address: String) -> Self {
        Self { address }
    }

    pub fn address(&self) -> String {
        self.address.clone()
    }
}
