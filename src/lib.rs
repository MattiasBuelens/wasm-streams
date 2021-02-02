//! Working with the Web [Streams API](https://developer.mozilla.org/en-US/docs/Web/API/Streams_API)
//! in Rust.
//!
//! This crate provides wrappers around [`ReadableStream`](crate::ReadableStream),
//! [`WritableStream`](crate::WritableStream) and [`TransformStream`](crate::TransformStream).
//! It also supports converting from and into [`Stream`](futures::Stream)s
//! and [`Sink`](futures::Sink)s from the [futures crate](futures).

pub use readable::ReadableStream;
pub use transform::TransformStream;
pub use writable::WritableStream;

pub(crate) mod queuing_strategy;
pub mod readable;
pub mod transform;
pub(crate) mod util;
pub mod writable;
