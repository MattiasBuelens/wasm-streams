use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug)]
pub(crate) struct QueuingStrategy {
    high_water_mark: f64,
}

impl QueuingStrategy {
    pub fn new(high_water_mark: f64) -> Self {
        Self { high_water_mark }
    }
}

#[wasm_bindgen]
impl QueuingStrategy {
    #[wasm_bindgen(getter, js_name = highWaterMark)]
    pub fn high_water_mark(&self) -> f64 {
        self.high_water_mark
    }
}
