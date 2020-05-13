use js_sys::Promise;
use wasm_bindgen::prelude::*;

use crate::queuing_strategy::QueuingStrategy;

use super::into_underlying_sink::IntoUnderlyingSink;

#[wasm_bindgen]
extern "C" {
    #[derive(Clone, Debug)]
    pub type WritableStream;

    #[wasm_bindgen(constructor)]
    pub fn new() -> WritableStream;

    #[wasm_bindgen(constructor)]
    pub(crate) fn new_with_sink(
        sink: IntoUnderlyingSink,
        strategy: QueuingStrategy,
    ) -> WritableStream;

    #[wasm_bindgen(method, getter, js_name = locked)]
    pub fn is_locked(this: &WritableStream) -> bool;

    #[wasm_bindgen(method, js_name = abort)]
    pub fn abort(this: &WritableStream) -> Promise;

    #[wasm_bindgen(method, js_name = abort)]
    pub fn abort_with_reason(this: &WritableStream, reason: &JsValue) -> Promise;

    #[wasm_bindgen(method, catch, js_name = getWriter)]
    pub fn get_writer(this: &WritableStream) -> Result<WritableStreamDefaultWriter, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[derive(Clone, Debug)]
    pub type WritableStreamDefaultWriter;

    #[wasm_bindgen(method, getter, js_name = closed)]
    pub fn closed(this: &WritableStreamDefaultWriter) -> Promise;

    #[wasm_bindgen(method, getter, js_name = desiredSize)]
    pub fn desired_size(this: &WritableStreamDefaultWriter) -> Option<f64>;

    #[wasm_bindgen(method, getter, js_name = ready)]
    pub fn ready(this: &WritableStreamDefaultWriter) -> Promise;

    #[wasm_bindgen(method, js_name = abort)]
    pub fn abort(this: &WritableStreamDefaultWriter) -> Promise;

    #[wasm_bindgen(method, js_name = abort)]
    pub fn abort_with_reason(this: &WritableStreamDefaultWriter, reason: &JsValue) -> Promise;

    #[wasm_bindgen(method, js_name = close)]
    pub fn close(this: &WritableStreamDefaultWriter) -> Promise;

    #[wasm_bindgen(method, js_name = write)]
    pub fn write(this: &WritableStreamDefaultWriter, chunk: JsValue) -> Promise;

    #[wasm_bindgen(method, catch, js_name = releaseLock)]
    pub fn release_lock(this: &WritableStreamDefaultWriter) -> Result<(), JsValue>;
}
