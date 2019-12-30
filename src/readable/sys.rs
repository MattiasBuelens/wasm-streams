use js_sys::Promise;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[derive(Clone, Debug)]
    pub type ReadableStream;

    #[wasm_bindgen(constructor)]
    pub fn new() -> ReadableStream;

    #[wasm_bindgen(method, getter, js_name = locked)]
    pub fn is_locked(this: &ReadableStream) -> bool;

    #[wasm_bindgen(method, js_name = cancel)]
    pub fn cancel(this: &ReadableStream) -> Promise;

    #[wasm_bindgen(method, js_name = cancel)]
    pub fn cancel_with_reason(this: &ReadableStream, reason: &JsValue) -> Promise;

    #[wasm_bindgen(method, catch, js_name = getReader)]
    pub fn get_reader(this: &ReadableStream) -> Result<ReadableStreamDefaultReader, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[derive(Clone, Debug)]
    pub type ReadableStreamDefaultReader;

    #[wasm_bindgen(method, getter, js_name = closed)]
    pub fn closed(this: &ReadableStreamDefaultReader) -> Promise;

    #[wasm_bindgen(method, js_name = cancel)]
    pub fn cancel(this: &ReadableStreamDefaultReader) -> Promise;

    #[wasm_bindgen(method, js_name = cancel)]
    pub fn cancel_with_reason(this: &ReadableStreamDefaultReader, reason: &JsValue) -> Promise;

    #[wasm_bindgen(method, js_name = read)]
    pub fn read(this: &ReadableStreamDefaultReader) -> Promise;

    #[wasm_bindgen(method, catch, js_name = releaseLock)]
    pub fn release_lock(this: &ReadableStreamDefaultReader) -> Result<(), JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[derive(Clone, Debug)]
    pub type ReadableStreamReadResult;

    #[wasm_bindgen(method, getter, js_name = done)]
    pub fn is_done(this: &ReadableStreamReadResult) -> bool;

    #[wasm_bindgen(method, getter, js_name = value)]
    pub fn value(this: &ReadableStreamReadResult) -> JsValue;
}
