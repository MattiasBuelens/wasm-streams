//! Raw bindings to JavaScript objects used
//! by a [`WritableStream`](https://developer.mozilla.org/en-US/docs/Web/API/WritableStream).
//! These are re-exported from [web-sys](https://docs.rs/web-sys/0.3.70/web_sys/struct.WritableStream.html).
use js_sys::Error;
use wasm_bindgen::prelude::*;
pub use web_sys::WritableStream;
pub use web_sys::WritableStreamDefaultWriter;

use crate::writable::into_underlying_sink::IntoUnderlyingSink;

#[wasm_bindgen]
extern "C" {
    /// A raw [`WritableStream`](https://developer.mozilla.org/en-US/docs/Web/API/WritableStream).
    #[wasm_bindgen(js_name = WritableStream, typescript_type = "WritableStream")]
    #[derive(Clone, Debug)]
    pub(crate) type WritableStreamExt;

    #[wasm_bindgen(constructor, js_class = WritableStream)]
    pub(crate) fn new_with_into_underlying_sink(sink: IntoUnderlyingSink) -> WritableStreamExt;
}

#[wasm_bindgen]
extern "C" {
    /// Additional methods for [`WritableStreamDefaultWriter`](web_sys::WritableStreamDefaultWriter).
    pub(crate) type WritableStreamDefaultWriterExt;

    #[wasm_bindgen(method, catch, js_name = releaseLock)]
    pub(crate) fn try_release_lock(this: &WritableStreamDefaultWriterExt) -> Result<(), Error>;
}
