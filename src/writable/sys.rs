//! Raw bindings to JavaScript objects used
//! by a [`WritableStream`](https://developer.mozilla.org/en-US/docs/Web/API/WritableStream).
use js_sys::{Error, Promise};
use wasm_bindgen::prelude::*;

use super::into_underlying_sink::IntoUnderlyingSink;

#[wasm_bindgen]
extern "C" {
    /// A raw [`WritableStream`](https://developer.mozilla.org/en-US/docs/Web/API/WritableStream).
    #[wasm_bindgen(js_name = WritableStream, typescript_type = "WritableStream")]
    #[derive(Clone, Debug)]
    pub type WritableStream;

    #[wasm_bindgen(constructor)]
    pub fn new() -> WritableStream;

    #[wasm_bindgen(constructor)]
    pub(crate) fn new_with_sink(sink: IntoUnderlyingSink) -> WritableStream;

    #[wasm_bindgen(method, getter, js_name = locked)]
    pub fn is_locked(this: &WritableStream) -> bool;

    #[wasm_bindgen(method, catch, js_name = abort)]
    pub async fn abort(this: &WritableStream) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name = abort)]
    pub async fn abort_with_reason(this: &WritableStream, reason: &JsValue) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name = getWriter)]
    pub fn get_writer(this: &WritableStream) -> Result<WritableStreamDefaultWriter, Error>;
}

#[wasm_bindgen]
extern "C" {
    /// A raw [`WritableStreamDefaultWriter`](https://developer.mozilla.org/en-US/docs/Web/API/WritableStreamDefaultWriter).
    #[derive(Clone, Debug)]
    pub type WritableStreamDefaultWriter;

    #[wasm_bindgen(method, getter, catch, js_name = closed)]
    pub async fn closed(this: &WritableStreamDefaultWriter) -> Result<(), JsValue>;

    #[wasm_bindgen(method, getter, js_name = desiredSize)]
    pub fn desired_size(this: &WritableStreamDefaultWriter) -> Option<f64>;

    #[wasm_bindgen(method, getter, js_name = ready)]
    pub fn ready(this: &WritableStreamDefaultWriter) -> Promise;

    #[wasm_bindgen(method, catch, js_name = abort)]
    pub async fn abort(this: &WritableStreamDefaultWriter) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name = abort)]
    pub async fn abort_with_reason(
        this: &WritableStreamDefaultWriter,
        reason: &JsValue,
    ) -> Result<(), JsValue>;

    #[wasm_bindgen(method, js_name = close)]
    pub fn close(this: &WritableStreamDefaultWriter) -> Promise;

    #[wasm_bindgen(method, js_name = write)]
    pub fn write(this: &WritableStreamDefaultWriter, chunk: JsValue) -> Promise;

    #[wasm_bindgen(method, js_name = releaseLock)]
    pub fn release_lock(this: &WritableStreamDefaultWriter);
}
