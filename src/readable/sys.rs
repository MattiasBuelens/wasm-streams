//! Raw bindings to JavaScript objects used
//! by a [`ReadableStream`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream).
//! These are re-exported from [web-sys](https://docs.rs/web-sys/0.3.64/web_sys/struct.ReadableStream.html).
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
pub use web_sys::ReadableByteStreamController;
// Re-export from web-sys
pub use web_sys::ReadableStream;
pub use web_sys::ReadableStreamByobReader as ReadableStreamBYOBReader;
pub use web_sys::ReadableStreamByobRequest as ReadableStreamBYOBRequest;
pub use web_sys::ReadableStreamDefaultController;
pub use web_sys::ReadableStreamDefaultReader;
pub use web_sys::ReadableStreamGetReaderOptions;
pub use web_sys::ReadableStreamReaderMode;
pub use web_sys::StreamPipeOptions as PipeOptions;

#[wasm_bindgen]
extern "C" {
    /// A result returned by [`ReadableStreamDefaultReader.read`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamDefaultReader/read).
    #[derive(Clone, Debug)]
    pub(crate) type ReadableStreamDefaultReadResult;

    #[wasm_bindgen(method, getter, js_name = done)]
    pub(crate) fn is_done(this: &ReadableStreamDefaultReadResult) -> bool;

    #[wasm_bindgen(method, getter, js_name = value)]
    pub(crate) fn value(this: &ReadableStreamDefaultReadResult) -> JsValue;
}

#[wasm_bindgen]
extern "C" {
    /// A result returned by [`ReadableStreamBYOBReader.read`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamBYOBReader/read).
    #[derive(Clone, Debug)]
    pub(crate) type ReadableStreamBYOBReadResult;

    #[wasm_bindgen(method, getter, js_name = done)]
    pub(crate) fn is_done(this: &ReadableStreamBYOBReadResult) -> bool;

    #[wasm_bindgen(method, getter, js_name = value)]
    pub(crate) fn value(this: &ReadableStreamBYOBReadResult) -> Option<Uint8Array>;
}
