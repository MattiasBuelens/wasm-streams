//! Bindings and conversions for
//! [writable streams](https://developer.mozilla.org/en-US/docs/Web/API/WritableStream).
use std::future::Future;
use std::marker::PhantomData;

use futures::Sink;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

pub use into_sink::IntoSink;

use crate::writable::into_underlying_sink::IntoUnderlyingSink;

mod into_sink;
mod into_underlying_sink;
pub mod sys;

/// A [`WritableStream`](https://developer.mozilla.org/en-US/docs/Web/API/WritableStream).
///
/// `WritableStream`s can be created from a [raw JavaScript stream](sys::WritableStream) with
/// [`from_raw`](Self::from_raw), or from a Rust [`Sink`](Sink)
/// with [`from_sink`](Self::from_sink).
///
/// They can be converted into a [raw JavaScript stream](sys::WritableStream) with
/// [`into_raw`](Self::into_raw), or into a Rust [`Sink`](Sink)
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

    /// Creates a new `WritableStream` from a [`Sink`](Sink).
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
    pub fn is_locked(&self) -> bool {
        self.raw.is_locked()
    }

    /// [Aborts](https://streams.spec.whatwg.org/#abort-a-writable-stream) the stream,
    /// signaling that the producer can no longer successfully write to the stream
    /// and it is to be immediately moved to an errored state, with any queued-up writes discarded.
    ///
    /// If the stream is currently locked to a writer, then this returns an error.
    pub fn abort<'a>(&'a mut self) -> impl Future<Output = Result<(), JsValue>> + 'a {
        let promise = self.raw.abort();
        async {
            let js_value = JsFuture::from(promise).await?;
            debug_assert!(js_value.is_undefined());
            Ok(())
        }
    }

    /// [Aborts](https://streams.spec.whatwg.org/#abort-a-writable-stream) the stream with the
    /// given `reason`, signaling that the producer can no longer successfully write to the stream
    /// and it is to be immediately moved to an errored state, with any queued-up writes discarded.
    ///
    /// If the stream is currently locked to a writer, then this returns an error.
    pub fn abort_with_reason<'a>(
        &'a mut self,
        reason: &JsValue,
    ) -> impl Future<Output = Result<(), JsValue>> + 'a {
        let promise = self.raw.abort_with_reason(reason);
        async {
            let js_value = JsFuture::from(promise).await?;
            debug_assert!(js_value.is_undefined());
            Ok(())
        }
    }

    /// Creates a [writer](WritableStreamDefaultWriter) and
    /// [locks](https://streams.spec.whatwg.org/#lock) the stream to the new writer.
    ///
    /// While the stream is locked, no other writer can be acquired until this one is released.
    ///
    /// If the stream is already locked to a writer, then this returns an error.
    pub fn get_writer(&mut self) -> Result<WritableStreamDefaultWriter, js_sys::Error> {
        Ok(WritableStreamDefaultWriter {
            raw: self.raw.get_writer()?,
            _stream: PhantomData,
        })
    }

    /// Converts this `WritableStream` into a [`Sink`](Sink).
    ///
    /// Items and errors are represented by their raw [`JsValue`](JsValue).
    /// Use [`with`](futures::SinkExt::with) and/or [`sink_map_err`](futures::SinkExt::sink_map_err)
    /// on the returned stream to convert them to a more appropriate type.
    ///
    /// If the stream is already locked to a writer, then this returns an error
    /// along with the original `WritableStream`.
    pub fn into_sink(self) -> Result<IntoSink<'static>, (js_sys::Error, Self)> {
        let raw_writer = match self.raw.get_writer() {
            Ok(raw_writer) => raw_writer,
            Err(err) => return Err((err, self)),
        };
        let writer = WritableStreamDefaultWriter {
            raw: raw_writer,
            _stream: PhantomData,
        };
        Ok(writer.into_sink())
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

/// A [`WritableStreamDefaultWriter`](https://developer.mozilla.org/en-US/docs/Web/API/WritableStreamDefaultWriter)
/// that can be used to write chunks to a [`WritableStream`](WritableStream).
///
/// This is returned by the [`get_writer`](WritableStream::get_writer) method.
///
/// When the writer is dropped, it automatically [releases its lock](https://streams.spec.whatwg.org/#release-a-lock).
#[derive(Debug)]
pub struct WritableStreamDefaultWriter<'stream> {
    raw: sys::WritableStreamDefaultWriter,
    _stream: PhantomData<&'stream mut WritableStream>,
}

impl<'stream> WritableStreamDefaultWriter<'stream> {
    /// Acquires a reference to the underlying [JavaScript writer](sys::WritableStreamDefaultWriter).
    #[inline]
    pub fn as_raw(&self) -> &sys::WritableStreamDefaultWriter {
        &self.raw
    }

    /// Waits for the stream to become closed.
    ///
    /// This returns an error if the stream ever errors, or if the writer's lock is
    /// [released](https://streams.spec.whatwg.org/#release-a-lock) before the stream finishes
    /// closing.
    pub async fn closed(&self) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.as_raw().closed()).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    /// Returns the desired size to fill the stream's internal queue.
    ///
    /// * It can be negative, if the queue is over-full.
    ///   A producer can use this information to determine the right amount of data to write.
    /// * It will be `None` if the stream cannot be successfully written to
    ///   (due to either being errored, or having an abort queued up).
    /// * It will return zero if the stream is closed.
    pub fn desired_size(&self) -> Option<f64> {
        self.as_raw().desired_size()
    }

    /// Waits until the desired size to fill the stream's internal queue transitions
    /// from non-positive to positive, signaling that it is no longer applying backpressure.
    ///
    /// Once the desired size to fill the stream's internal queue dips back to zero or below,
    /// this will return a new future that stays pending until the next transition.
    ///
    /// This returns an error if the stream ever errors, or if the writer's lock is
    /// [released](https://streams.spec.whatwg.org/#release-a-lock) before the stream finishes
    /// closing.
    pub async fn ready(&self) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.as_raw().ready()).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    /// [Aborts](https://streams.spec.whatwg.org/#abort-a-writable-stream) the stream,
    /// signaling that the producer can no longer successfully write to the stream.
    ///
    /// Equivalent to [`WritableStream.abort`](WritableStream::abort).
    pub fn abort<'a>(&'a mut self) -> impl Future<Output = Result<(), JsValue>> + 'a {
        let promise = self.as_raw().abort();
        async {
            let js_value = JsFuture::from(promise).await?;
            debug_assert!(js_value.is_undefined());
            Ok(())
        }
    }

    /// [Aborts](https://streams.spec.whatwg.org/#abort-a-writable-stream) the stream with the
    /// given `reason`, signaling that the producer can no longer successfully write to the stream.
    ///
    /// Equivalent to [`WritableStream.abort_with_reason`](WritableStream::abort_with_reason).
    pub fn abort_with_reason<'a>(
        &'a mut self,
        reason: &JsValue,
    ) -> impl Future<Output = Result<(), JsValue>> + 'a {
        let promise = self.as_raw().abort_with_reason(reason);
        async {
            let js_value = JsFuture::from(promise).await?;
            debug_assert!(js_value.is_undefined());
            Ok(())
        }
    }

    /// Writes the given `chunk` to the writable stream, by waiting until any previous writes
    /// have finished successfully, and then sending the chunk to the underlying sink's `write()`
    /// method.
    ///
    /// This returns `Ok(())` upon a successful write, or `Err(error)` if the write fails or stream
    /// becomes errored before the writing process is initiated.
    ///
    /// Note that what "success" means is up to the underlying sink; it might indicate simply
    /// that the chunk has been accepted, and not necessarily that it is safely saved to
    /// its ultimate destination.
    pub fn write<'a>(
        &'a mut self,
        chunk: JsValue,
    ) -> impl Future<Output = Result<(), JsValue>> + 'a {
        let promise = self.as_raw().write(chunk);
        async {
            let js_value = JsFuture::from(promise).await?;
            debug_assert!(js_value.is_undefined());
            Ok(())
        }
    }

    /// Closes the stream.
    ///
    /// The underlying sink will finish processing any previously-written chunks, before invoking
    /// its close behavior. During this time any further attempts to write will fail
    /// (without erroring the stream).
    ///
    /// This returns `Ok(())` if all remaining chunks are successfully written and the stream
    /// successfully closes, or `Err(error)` if an error is encountered during this process.
    pub fn close<'a>(&'a mut self) -> impl Future<Output = Result<(), JsValue>> + 'a {
        let promise = self.as_raw().close();
        async {
            let js_value = JsFuture::from(promise).await?;
            debug_assert!(js_value.is_undefined());
            Ok(())
        }
    }

    /// Converts this `WritableStreamDefaultWriter` into a [`Sink`](Sink).
    ///
    /// This is similar to [`WritableStream.into_sink`](WritableStream::into_sink),
    /// except that after the returned `Sink` is dropped, the original `WritableStream` is still
    /// usable. This allows writing only a few chunks through the `Sink`, while still allowing
    /// another writer to write more chunks later on.
    pub fn into_sink(self) -> IntoSink<'stream> {
        IntoSink::new(self)
    }
}

impl Drop for WritableStreamDefaultWriter<'_> {
    fn drop(&mut self) {
        self.as_raw().release_lock()
    }
}
