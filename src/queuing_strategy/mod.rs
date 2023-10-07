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

    #[cfg(web_sys_unstable_apis)]
    pub fn from_raw(raw: web_sys::QueuingStrategy) -> Self {
        let high_water_mark = js_sys::Reflect::get(&raw, &JsValue::from_str("highWaterMark"))
            .unwrap_throw()
            .as_f64()
            .unwrap_throw();
        Self::new(high_water_mark)
    }

    #[cfg(web_sys_unstable_apis)]
    pub fn into_raw(self) -> web_sys::QueuingStrategy {
        let mut raw = web_sys::QueuingStrategy::new();
        raw.high_water_mark(self.high_water_mark);
        raw
    }
}

#[wasm_bindgen]
impl QueuingStrategy {
    #[wasm_bindgen(getter, js_name = highWaterMark)]
    pub fn high_water_mark(&self) -> f64 {
        self.high_water_mark
    }
}
