use js_sys::Object;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, js_name = QueuingStrategy)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) type QueuingStrategy;

    #[wasm_bindgen(method, getter, js_name = highWaterMark)]
    pub fn high_water_mark(this: &QueuingStrategy) -> f64;

    #[wasm_bindgen(method, setter, js_name = highWaterMark)]
    pub fn set_high_water_mark(this: &QueuingStrategy, value: f64);
}

impl QueuingStrategy {
    pub fn new(high_water_mark: f64) -> Self {
        let strategy = Object::new().unchecked_into::<QueuingStrategy>();
        strategy.set_high_water_mark(high_water_mark);
        strategy
    }

    #[cfg(web_sys_unstable_apis)]
    pub fn from_raw(raw: web_sys::QueuingStrategy) -> Self {
        raw.unchecked_into()
    }

    #[cfg(web_sys_unstable_apis)]
    pub fn as_raw(&self) -> &web_sys::QueuingStrategy {
        self.unchecked_ref()
    }

    #[cfg(web_sys_unstable_apis)]
    pub fn into_raw(self) -> web_sys::QueuingStrategy {
        self.unchecked_into()
    }
}
