//! Raw bindings to JavaScript objects used
//! by a [`ReadableStream`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream).
//! These are re-exported from [web-sys](https://docs.rs/web-sys/0.3.64/web_sys/struct.ReadableStream.html).
use js_sys::{Array, Error, Object, Uint8Array};
use wasm_bindgen::prelude::*;
use web_sys::QueuingStrategy;
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

use crate::readable::into_underlying_byte_source::IntoUnderlyingByteSource;
use crate::readable::into_underlying_source::IntoUnderlyingSource;

#[wasm_bindgen]
extern "C" {
    /// Additional methods for [`ReadableStream`](web_sys::ReadableStream).
    #[wasm_bindgen(js_name = ReadableStream, typescript_type = "ReadableStream")]
    pub(crate) type ReadableStreamExt;

    #[wasm_bindgen(constructor, catch, js_class = ReadableStream)]
    pub(crate) fn new_with_into_underlying_source(
        source: IntoUnderlyingSource,
        strategy: &QueuingStrategy,
    ) -> Result<ReadableStreamExt, Error>;

    #[wasm_bindgen(constructor, catch, js_class = ReadableStream)]
    pub(crate) fn new_with_into_underlying_byte_source(
        source: IntoUnderlyingByteSource,
    ) -> Result<ReadableStreamExt, Error>;

    #[wasm_bindgen(method, catch, js_class = ReadableStream, js_name = getReader)]
    pub(crate) fn try_get_reader(this: &ReadableStreamExt) -> Result<Object, Error>;

    #[wasm_bindgen(method, catch, js_class = ReadableStream, js_name = getReader)]
    pub(crate) fn try_get_reader_with_options(
        this: &ReadableStreamExt,
        options: &ReadableStreamGetReaderOptions,
    ) -> Result<Object, Error>;

    #[wasm_bindgen(method, catch, js_class = ReadableStream, js_name = tee)]
    pub(crate) fn try_tee(this: &ReadableStreamExt) -> Result<Array, Error>;
}

#[wasm_bindgen]
extern "C" {
    /// Additional methods for [`ReadableStreamDefaultReader`](web_sys::ReadableStreamDefaultReader)
    /// and [`ReadableStreamByobReader`](web_sys::ReadableStreamByobReader).
    pub(crate) type ReadableStreamReaderExt;

    #[wasm_bindgen(method, catch, js_name = releaseLock)]
    pub(crate) fn try_release_lock(this: &ReadableStreamReaderExt) -> Result<(), Error>;
}

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
