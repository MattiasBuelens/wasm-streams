use wasm_bindgen::prelude::*;

use crate::readable::sys::ReadableStream;
use crate::writable::sys::WritableStream;

#[wasm_bindgen]
extern "C" {
    #[derive(Clone, Debug)]
    pub type TransformStream;

    #[wasm_bindgen(constructor)]
    pub fn new() -> TransformStream;

    #[wasm_bindgen(method, getter, js_name = readable)]
    pub fn readable(this: &TransformStream) -> ReadableStream;

    #[wasm_bindgen(method, getter, js_name = writable)]
    pub fn writable(this: &TransformStream) -> WritableStream;
}
