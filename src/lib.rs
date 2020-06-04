//! Working with the Web [Streams API](https://developer.mozilla.org/en-US/docs/Web/API/Streams_API)
//! in Rust.
//!
//! This crate provides wrappers around [`ReadableStream`](crate::ReadableStream),
//! [`WritableStream`](crate::WritableStream) and [`TransformStream`](crate::TransformStream).
//! It also supports converting from and into [`Stream`](futures::Stream)s
//! and [`Sink`](futures::Sink)s from the [futures crate](https://docs.rs/futures/).
//!
//! ## Reading a streaming fetch response
//!
//! This example makes an HTTP request using `fetch()` from [web-sys](https://docs.rs/web-sys/),
//! and then consumes the response body as a Rust `Stream`.
//!
//! ```no_run
//! use futures::StreamExt;
//! use wasm_bindgen::JsCast;
//! use wasm_bindgen_futures::JsFuture;
//! use wasm_streams::ReadableStream;
//! use web_sys::{console, window, Response};
//!
//! async fn fetch_example(url: &str) {
//!     // Make a fetch request
//!     let window = window().unwrap();
//!     let resp_value = JsFuture::from(window.fetch_with_str(url)).await.expect("fetch failed");
//!     let resp: Response = resp_value.dyn_into().unwrap();
//!
//!     // Get the response's body as a ReadableStream
//!     let raw_body = resp.body().unwrap();
//!     let body = ReadableStream::from_raw(raw_body.unchecked_into());
//!
//!     // Convert the ReadableStream to a Rust stream
//!     let mut stream = body.into_stream().unwrap();
//!
//!     // Consume the stream
//!     while let Some(Ok(chunk)) = stream.next().await {
//!         console::log_1(&chunk);
//!     }
//! }
//! ```

pub(crate) mod queuing_strategy;
pub mod readable;
pub mod transform;
pub mod writable;

pub use readable::ReadableStream;
pub use transform::TransformStream;
pub use writable::WritableStream;
