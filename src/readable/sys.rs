//! Raw bindings to JavaScript objects used
//! by a [`ReadableStream`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream).
use js_sys::{Array, Error, Promise};
use wasm_bindgen::prelude::*;
use web_sys::AbortSignal;

use crate::queuing_strategy::QueuingStrategy;
use crate::writable::sys::WritableStream;

use super::into_underlying_source::IntoUnderlyingSource;

#[wasm_bindgen]
extern "C" {
    /// A raw [`ReadableStream`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream).
    ///
    /// This represents the same JavaScript objects as [`web_sys::ReadableStream`][web-sys].
    /// If you're using an API that returns such an object, you can cast it to this type using
    /// [`unchecked_into`][wasm_bindgen::JsCast::unchecked_into].
    ///
    /// [web-sys]: https://docs.rs/web-sys/latest/web_sys/struct.ReadableStream.html
    #[wasm_bindgen(js_name = ReadableStream, typescript_type = "ReadableStream")]
    #[derive(Clone, Debug)]
    pub type ReadableStream;

    #[wasm_bindgen(constructor)]
    pub fn new() -> ReadableStream;

    #[wasm_bindgen(constructor)]
    pub(crate) fn new_with_source(
        source: IntoUnderlyingSource,
        strategy: QueuingStrategy,
    ) -> ReadableStream;

    #[wasm_bindgen(method, getter, js_name = locked)]
    pub fn is_locked(this: &ReadableStream) -> bool;

    #[wasm_bindgen(method, js_name = cancel)]
    pub fn cancel(this: &ReadableStream) -> Promise;

    #[wasm_bindgen(method, js_name = cancel)]
    pub fn cancel_with_reason(this: &ReadableStream, reason: &JsValue) -> Promise;

    #[wasm_bindgen(method, catch, js_name = getReader)]
    pub fn get_reader(this: &ReadableStream) -> Result<ReadableStreamDefaultReader, Error>;

    #[wasm_bindgen(method, js_name = pipeTo)]
    pub fn pipe_to(this: &ReadableStream, dest: &WritableStream, opts: PipeOptions) -> Promise;

    #[wasm_bindgen(method, catch, js_name = tee)]
    pub fn tee(this: &ReadableStream) -> Result<Array, Error>;
}

#[wasm_bindgen]
extern "C" {
    /// A raw [`ReadableStreamDefaultController`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamDefaultController).
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
    /// A raw [`ReadableStreamDefaultReader`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamDefaultReader).
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
    pub fn release_lock(this: &ReadableStreamDefaultReader) -> Result<(), Error>;
}

#[wasm_bindgen]
extern "C" {
    /// A result returned by [`ReadableStreamDefaultReader.read`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamDefaultReader/read).
    #[derive(Clone, Debug)]
    pub type ReadableStreamReadResult;

    #[wasm_bindgen(method, getter, js_name = done)]
    pub fn is_done(this: &ReadableStreamReadResult) -> bool;

    #[wasm_bindgen(method, getter, js_name = value)]
    pub fn value(this: &ReadableStreamReadResult) -> JsValue;
}

/// Raw options for [`pipeTo()`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/pipeTo).
#[wasm_bindgen]
#[derive(Clone, Debug, Default)]
pub struct PipeOptions {
    prevent_close: bool,
    prevent_cancel: bool,
    prevent_abort: bool,
    signal: Option<AbortSignal>,
}

impl PipeOptions {
    pub fn new(
        prevent_close: bool,
        prevent_cancel: bool,
        prevent_abort: bool,
        signal: Option<AbortSignal>,
    ) -> Self {
        Self {
            prevent_close,
            prevent_cancel,
            prevent_abort,
            signal,
        }
    }
}

#[wasm_bindgen]
impl PipeOptions {
    #[wasm_bindgen(getter, js_name = preventClose)]
    pub fn prevent_close(&self) -> bool {
        self.prevent_close
    }

    #[wasm_bindgen(getter, js_name = preventCancel)]
    pub fn prevent_cancel(&self) -> bool {
        self.prevent_cancel
    }

    #[wasm_bindgen(getter, js_name = preventAbort)]
    pub fn prevent_abort(&self) -> bool {
        self.prevent_abort
    }

    #[wasm_bindgen(getter, js_name = signal)]
    pub fn signal(&self) -> Option<AbortSignal> {
        self.signal.clone()
    }
}
