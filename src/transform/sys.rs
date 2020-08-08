//! Raw bindings to JavaScript objects used
//! by a [`TransformStream`](https://developer.mozilla.org/en-US/docs/Web/API/TransformStream).
use wasm_bindgen::prelude::*;

use crate::readable::sys::ReadableStream;
use crate::writable::sys::WritableStream;

#[wasm_bindgen]
extern "C" {
    /// A raw [`TransformStream`](https://developer.mozilla.org/en-US/docs/Web/API/TransformStream).
    #[wasm_bindgen(js_name = TransformStream, typescript_type = "TransformStream")]
    #[derive(Clone, Debug)]
    pub type TransformStream;

    #[wasm_bindgen(constructor)]
    pub fn new() -> TransformStream;

    #[wasm_bindgen(method, getter, js_name = readable)]
    pub fn readable(this: &TransformStream) -> ReadableStream;

    #[wasm_bindgen(method, getter, js_name = writable)]
    pub fn writable(this: &TransformStream) -> WritableStream;
}
