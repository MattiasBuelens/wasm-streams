//! Links two copies of `wasm-streams` into a single `cdylib`, and touches every
//! type that carries `#[wasm_bindgen]` glue in both of them.
//!
//! On a version of `wasm-streams` that exports `IntoUnderlyingSource`,
//! `IntoUnderlyingByteSource` and `IntoUnderlyingSink` as `#[wasm_bindgen]`
//! classes, both copies emit the same `__wbg_*_free` symbols and the link fails
//! with `rust-lld: error: duplicate symbol: __wbg_intounderlyingbytesource_free`.
//!
//! Run with `cargo build --target wasm32-unknown-unknown` from this directory:
//! the test passes if it links. This is also wired up as an integration test
//! in `../tests/duplicate_symbols.rs`, so `cargo test` on the parent crate
//! covers it too.
//!
//! See <https://github.com/cloudflare/workers-rs/issues/1026>.

use futures_util::io::empty;
use futures_util::{SinkExt, sink, stream};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn old_streams() -> Vec<JsValue> {
    let readable = wasm_streams_old::ReadableStream::from_stream(stream::empty());
    let byte_readable = wasm_streams_old::ReadableStream::from_async_read(empty(), 1024);
    let writable = wasm_streams_old::WritableStream::from_sink(
        sink::drain().sink_map_err(|_| JsValue::UNDEFINED),
    );
    vec![
        readable.into_raw().into(),
        byte_readable.into_raw().into(),
        writable.into_raw().into(),
    ]
}

#[wasm_bindgen]
pub fn new_streams() -> Vec<JsValue> {
    let readable = wasm_streams_new::ReadableStream::from_stream(stream::empty());
    let byte_readable = wasm_streams_new::ReadableStream::from_async_read(empty(), 1024);
    let writable = wasm_streams_new::WritableStream::from_sink(
        sink::drain().sink_map_err(|_| JsValue::UNDEFINED),
    );
    vec![
        readable.into_raw().into(),
        byte_readable.into_raw().into(),
        writable.into_raw().into(),
    ]
}
