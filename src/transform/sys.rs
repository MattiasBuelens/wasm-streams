use js_sys::Promise;
use wasm_bindgen::prelude::*;

use crate::readable::sys::ReadableStream;
use crate::writable::sys::WritableStream;

#[wasm_bindgen]
extern "C" {
    #[derive(Clone, Debug)]
    pub type TransformStream;

    #[wasm_bindgen(constructor)]
    pub fn new() -> TransformStream;

    #[wasm_bindgen(constructor)]
    pub fn new_with_transformer(source: &Transformer) -> TransformStream;

    #[wasm_bindgen(method, getter, js_name = readable)]
    pub fn readable(this: &TransformStream) -> ReadableStream;

    #[wasm_bindgen(method, getter, js_name = writable)]
    pub fn writable(this: &TransformStream) -> WritableStream;
}

#[wasm_bindgen]
extern "C" {
    #[derive(Clone, Debug)]
    pub type TransformStreamDefaultController;

    #[wasm_bindgen(method, getter, js_name = desiredSize)]
    pub fn desired_size(this: &TransformStreamDefaultController) -> Option<f64>;

    #[wasm_bindgen(method, js_name = enqueue)]
    pub fn enqueue(this: &TransformStreamDefaultController, chunk: &JsValue);

    #[wasm_bindgen(method, js_name = error)]
    pub fn error(this: &TransformStreamDefaultController, error: &JsValue);

    #[wasm_bindgen(method, js_name = terminate)]
    pub fn terminate(this: &TransformStreamDefaultController);
}

#[wasm_bindgen]
extern "C" {
    #[derive(Clone, Debug)]
    pub type Transformer;

    #[wasm_bindgen(method, structural, setter, js_name = start)]
    pub fn set_start(this: &Transformer, cb: &Closure<dyn FnMut(TransformStreamDefaultController) -> Promise>);

    #[wasm_bindgen(method, structural, setter, js_name = transform)]
    pub fn set_transform(this: &Transformer, cb: &Closure<dyn FnMut(JsValue, TransformStreamDefaultController) -> Promise>);

    #[wasm_bindgen(method, structural, setter, js_name = flush)]
    pub fn set_flush(this: &Transformer, cb: &Closure<dyn FnMut(TransformStreamDefaultController) -> Promise>);
}
