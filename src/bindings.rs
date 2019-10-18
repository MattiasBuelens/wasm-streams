use js_sys::{Object, Promise};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    #[derive(Clone, Debug)]
    pub type ReadableStream;

    #[wasm_bindgen(constructor)]
    pub fn new() -> ReadableStream;

    #[wasm_bindgen(constructor)]
    pub fn new_with_source(source: &UnderlyingSource) -> ReadableStream;

    #[wasm_bindgen(method, getter, js_name = locked)]
    pub fn locked(this: &ReadableStream) -> bool;

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
    pub type ReadableStreamDefaultController;

    #[wasm_bindgen(method, getter, js_name = desiredSize)]
    pub fn desired_size(this: &ReadableStreamDefaultController) -> Option<f64>;

    #[wasm_bindgen(method, js_name = close)]
    pub fn close(this: &ReadableStreamDefaultController);

    #[wasm_bindgen(method, js_name = enqueue)]
    pub fn enqueue(this: &ReadableStreamDefaultController, chunk: &JsValue);

    #[wasm_bindgen(method, js_name = error)]
    pub fn error(this: &ReadableStreamDefaultController, error: &JsValue);
}

#[wasm_bindgen]
extern "C" {
    #[derive(Clone, Debug)]
    pub type UnderlyingSource;

    #[wasm_bindgen(method, structural, setter, js_name = start)]
    pub fn set_start(this: &UnderlyingSource, cb: &Closure<dyn FnMut(&ReadableStreamDefaultController)>);

    #[wasm_bindgen(method, structural, setter, js_name = pull)]
    pub fn set_pull(this: &UnderlyingSource, cb: &Closure<dyn FnMut(&ReadableStreamDefaultController)>);

    #[wasm_bindgen(method, structural, setter, js_name = cancel)]
    pub fn set_cancel(this: &UnderlyingSource, cb: &Closure<dyn FnMut(&JsValue)>);
}

impl UnderlyingSource {
    pub fn new() -> UnderlyingSource {
        UnderlyingSource::from(JsValue::from(Object::new()))
    }
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
    fn read_raw(this: &ReadableStreamDefaultReader) -> Promise;

    #[wasm_bindgen(method, catch, js_name = releaseLock)]
    pub fn release_lock(this: &ReadableStreamDefaultReader) -> Result<(), JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[derive(Clone, Debug)]
    type ReadableStreamReadResult;

    #[wasm_bindgen(method, getter, js_name = done)]
    fn done(this: &ReadableStreamReadResult) -> bool;

    #[wasm_bindgen(method, getter, js_name = value)]
    fn value(this: &ReadableStreamReadResult) -> JsValue;
}

impl ReadableStreamDefaultReader {
    pub async fn read(&self) -> Result<Option<JsValue>, JsValue> {
        let js_value = JsFuture::from(self.read_raw()).await?;
        let result = ReadableStreamReadResult::from(js_value);
        if result.done() {
            Ok(None)
        } else {
            Ok(Some(result.value()))
        }
    }
}