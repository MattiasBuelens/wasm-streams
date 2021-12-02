//! Bindings and conversions for
//! [writable streams](https://developer.mozilla.org/en-US/docs/Web/API/WritableStream).

use futures::Sink;
use wasm_bindgen::prelude::*;

pub use default_writer::WritableStreamDefaultWriter;
pub use into_async_write::IntoAsyncWrite;
pub use into_sink::IntoSink;
use into_underlying_sink::IntoUnderlyingSink;

use crate::util::promise_to_void_future;

mod default_writer;
mod into_async_write;
mod into_sink;
mod into_underlying_sink;
pub mod sys;

/// A [`WritableStream`](https://developer.mozilla.org/en-US/docs/Web/API/WritableStream).
///
/// `WritableStream`s can be created from a [raw JavaScript stream](sys::WritableStream) with
/// [`from_raw`](Self::from_raw), or from a Rust [`Sink`](futures::Sink)
/// with [`from_sink`](Self::from_sink).
///
/// They can be converted into a [raw JavaScript stream](sys::WritableStream) with
/// [`into_raw`](Self::into_raw), or into a Rust [`Sink`](futures::Sink)
/// with [`into_sink`](Self::into_sink).
#[derive(Debug)]
pub struct WritableStream {
    raw: sys::WritableStream,
}

impl WritableStream {
    /// Creates a new `WritableStream` from a [JavaScript stream](sys::WritableStream).
    #[inline]
    pub fn from_raw(raw: sys::WritableStream) -> Self {
        Self { raw }
    }

    /// Creates a new `WritableStream` from a [`Sink`](futures::Sink).
    ///
    /// Items and errors must be represented as raw [`JsValue`](JsValue)s.
    /// Use [`with`](futures::SinkExt::with) and/or [`sink_map_err`](futures::SinkExt::sink_map_err)
    /// to convert a sink's items to a `JsValue` before passing it to this function.
    pub fn from_sink<Si>(sink: Si) -> Self
    where
        Si: Sink<JsValue, Error = JsValue> + 'static,
    {
        let sink = IntoUnderlyingSink::new(Box::new(sink));
        // Use the default queuing strategy (with a HWM of 1 chunk).
        // We shouldn't set HWM to 0, since that would break piping to the writable stream.
        let raw = sys::WritableStream::new_with_sink(sink);
        WritableStream { raw }
    }

    /// Acquires a reference to the underlying [JavaScript stream](sys::WritableStream).
    #[inline]
    pub fn as_raw(&self) -> &sys::WritableStream {
        &self.raw
    }

    /// Consumes this `WritableStream`, returning the underlying [JavaScript stream](sys::WritableStream).
    #[inline]
    pub fn into_raw(self) -> sys::WritableStream {
        self.raw
    }

    /// Returns `true` if the stream is [locked to a writer](https://streams.spec.whatwg.org/#lock).
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.as_raw().is_locked()
    }

    /// [Aborts](https://streams.spec.whatwg.org/#abort-a-writable-stream) the stream,
    /// signaling that the producer can no longer successfully write to the stream
    /// and it is to be immediately moved to an errored state, with any queued-up writes discarded.
    ///
    /// If the stream is currently locked to a writer, then this returns an error.
    pub async fn abort(&mut self) -> Result<(), JsValue> {
        promise_to_void_future(self.as_raw().abort()).await
    }

    /// [Aborts](https://streams.spec.whatwg.org/#abort-a-writable-stream) the stream with the
    /// given `reason`, signaling that the producer can no longer successfully write to the stream
    /// and it is to be immediately moved to an errored state, with any queued-up writes discarded.
    ///
    /// If the stream is currently locked to a writer, then this returns an error.
    pub async fn abort_with_reason(&mut self, reason: &JsValue) -> Result<(), JsValue> {
        promise_to_void_future(self.as_raw().abort_with_reason(reason)).await
    }

    /// Creates a [writer](WritableStreamDefaultWriter) and
    /// [locks](https://streams.spec.whatwg.org/#lock) the stream to the new writer.
    ///
    /// While the stream is locked, no other writer can be acquired until this one is released.
    ///
    /// **Panics** if the stream is already locked to a writer. For a non-panicking variant,
    /// use [`try_get_writer`](Self::try_get_writer).
    #[inline]
    pub fn get_writer(&mut self) -> WritableStreamDefaultWriter {
        self.try_get_writer()
            .expect_throw("already locked to a writer")
    }

    /// Try to create a [writer](WritableStreamDefaultWriter) and
    /// [lock](https://streams.spec.whatwg.org/#lock) the stream to the new writer.
    ///
    /// While the stream is locked, no other writer can be acquired until this one is released.
    ///
    /// If the stream is already locked to a writer, then this returns an error.
    pub fn try_get_writer(&mut self) -> Result<WritableStreamDefaultWriter, js_sys::Error> {
        WritableStreamDefaultWriter::new(self)
    }

    /// Converts this `WritableStream` into a [`Sink`](futures::Sink).
    ///
    /// Items and errors are represented by their raw [`JsValue`](JsValue).
    /// Use [`with`](futures::SinkExt::with) and/or [`sink_map_err`](futures::SinkExt::sink_map_err)
    /// on the returned stream to convert them to a more appropriate type.
    ///
    /// **Panics** if the stream is already locked to a writer. For a non-panicking variant,
    /// use [`try_into_sink`](Self::try_into_sink).
    #[inline]
    pub fn into_sink(self) -> IntoSink<'static> {
        self.try_into_sink()
            .expect_throw("already locked to a writer")
    }

    /// Try to convert this `WritableStream` into a [`Sink`](futures::Sink).
    ///
    /// Items and errors are represented by their raw [`JsValue`](JsValue).
    /// Use [`with`](futures::SinkExt::with) and/or [`sink_map_err`](futures::SinkExt::sink_map_err)
    /// on the returned stream to convert them to a more appropriate type.
    ///
    /// If the stream is already locked to a writer, then this returns an error
    /// along with the original `WritableStream`.
    pub fn try_into_sink(mut self) -> Result<IntoSink<'static>, (js_sys::Error, Self)> {
        let writer = WritableStreamDefaultWriter::new(&mut self).map_err(|err| (err, self))?;
        Ok(writer.into_sink())
    }

    pub fn into_async_write(self) -> IntoAsyncWrite<'static> {
        self.try_into_async_write()
            .expect_throw("already locked to a writer")
    }

    pub fn try_into_async_write(self) -> Result<IntoAsyncWrite<'static>, (js_sys::Error, Self)> {
        Ok(IntoAsyncWrite::new(self.try_into_sink()?))
    }
}

impl<Si> From<Si> for WritableStream
where
    Si: Sink<JsValue, Error = JsValue> + 'static,
{
    /// Equivalent to [`from_sink`](Self::from_sink).
    #[inline]
    fn from(sink: Si) -> Self {
        Self::from_sink(sink)
    }
}
