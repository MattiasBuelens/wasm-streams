use std::marker::PhantomData;

use wasm_bindgen::{JsCast, JsValue, throw_val};

use super::{IntoAsyncWrite, IntoSink, WritableStream, sys};

/// A [`WritableStreamDefaultWriter`](https://developer.mozilla.org/en-US/docs/Web/API/WritableStreamDefaultWriter)
/// that can be used to write chunks to a [`WritableStream`](WritableStream).
///
/// This is returned by the [`get_writer`](WritableStream::get_writer) method.
///
/// When the writer is dropped, it automatically [releases its lock](https://streams.spec.whatwg.org/#release-a-lock).
/// This is best-effort: if `releaseLock()` throws, the error is ignored and the stream stays locked.
#[derive(Debug)]
pub struct WritableStreamDefaultWriter<'stream> {
    raw: sys::WritableStreamDefaultWriter,
    _stream: PhantomData<&'stream mut WritableStream>,
}

impl<'stream> WritableStreamDefaultWriter<'stream> {
    pub(crate) fn new(stream: &mut WritableStream) -> Result<Self, js_sys::Error> {
        Ok(Self {
            raw: stream.as_raw().get_writer()?,
            _stream: PhantomData,
        })
    }

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
        self.as_raw().closed().await.map(|_| ())
    }

    /// Returns the desired size to fill the stream's internal queue.
    ///
    /// * It can be negative, if the queue is over-full.
    ///   A producer can use this information to determine the right amount of data to write.
    /// * It will be `None` if the stream cannot be successfully written to
    ///   (due to either being errored, or having an abort queued up).
    /// * It will return zero if the stream is closed.
    #[inline]
    pub fn desired_size(&self) -> Option<f64> {
        self.as_raw()
            .desired_size()
            .unwrap_or_else(|error| throw_val(error))
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
        self.as_raw().ready().await.map(|_| ())
    }

    /// [Aborts](https://streams.spec.whatwg.org/#abort-a-writable-stream) the stream,
    /// signaling that the producer can no longer successfully write to the stream.
    ///
    /// Equivalent to [`WritableStream.abort`](WritableStream::abort).
    pub async fn abort(&mut self) -> Result<(), JsValue> {
        self.as_raw().abort().await.map(|_| ())
    }

    /// [Aborts](https://streams.spec.whatwg.org/#abort-a-writable-stream) the stream with the
    /// given `reason`, signaling that the producer can no longer successfully write to the stream.
    ///
    /// Equivalent to [`WritableStream.abort_with_reason`](WritableStream::abort_with_reason).
    pub async fn abort_with_reason(&mut self, reason: &JsValue) -> Result<(), JsValue> {
        self.as_raw().abort_with_reason(reason).await.map(|_| ())
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
    pub async fn write(&mut self, chunk: JsValue) -> Result<(), JsValue> {
        self.as_raw().write_with_chunk(&chunk).await.map(|_| ())
    }

    /// Closes the stream.
    ///
    /// The underlying sink will finish processing any previously-written chunks, before invoking
    /// its close behavior. During this time any further attempts to write will fail
    /// (without erroring the stream).
    ///
    /// This returns `Ok(())` if all remaining chunks are successfully written and the stream
    /// successfully closes, or `Err(error)` if an error is encountered during this process.
    pub async fn close(&mut self) -> Result<(), JsValue> {
        self.as_raw().close().await.map(|_| ())
    }

    /// [Releases](https://streams.spec.whatwg.org/#release-a-lock) this writer's lock on the
    /// corresponding stream.
    ///
    /// Per the Streams standard this always succeeds, but some hosts (e.g. Cloudflare Workers
    /// outside a request) may throw. For a non-throwing variant, use
    /// [`try_release_lock`](Self::try_release_lock).
    #[inline]
    pub fn release_lock(mut self) {
        if let Err(err) = self.try_release_lock_mut() {
            throw_val(err.into())
        }
    }

    /// Try to [release](https://streams.spec.whatwg.org/#release-a-lock) this writer's lock on the
    /// corresponding stream.
    ///
    /// If the host refuses to release the lock, this returns the error and leaves the writer
    /// locked to the stream.
    #[inline]
    pub fn try_release_lock(mut self) -> Result<(), (js_sys::Error, Self)> {
        self.try_release_lock_mut().map_err(|err| (err, self))
    }

    fn try_release_lock_mut(&mut self) -> Result<(), js_sys::Error> {
        self.as_raw()
            .unchecked_ref::<sys::WritableStreamDefaultWriterExt>()
            .try_release_lock()
    }

    /// Converts this `WritableStreamDefaultWriter` into a [`Sink`].
    ///
    /// This is similar to [`WritableStream.into_sink`](WritableStream::into_sink),
    /// except that after the returned `Sink` is dropped, the original `WritableStream` is still
    /// usable. This allows writing only a few chunks through the `Sink`, while still allowing
    /// another writer to write more chunks later on.
    ///
    /// [`Sink`]: https://docs.rs/futures/0.3.30/futures/sink/trait.Sink.html
    #[inline]
    pub fn into_sink(self) -> IntoSink<'stream> {
        IntoSink::new(self)
    }

    /// Converts this `WritableStreamDefaultWriter` into an [`AsyncWrite`].
    ///
    /// The writable stream must accept [`Uint8Array`](js_sys::Uint8Array) chunks.
    ///
    /// This is similar to [`WritableStream.into_async_write`](WritableStream::into_async_write),
    /// except that after the returned `AsyncWrite` is dropped, the original `WritableStream` is
    /// still usable. This allows writing only a few bytes through the `AsyncWrite`, while still
    /// allowing another writer to write more bytes later on.
    ///
    /// [`AsyncWrite`]: https://docs.rs/futures/0.3.30/futures/io/trait.AsyncWrite.html
    #[inline]
    pub fn into_async_write(self) -> IntoAsyncWrite<'stream> {
        IntoAsyncWrite::new(self.into_sink())
    }
}

impl Drop for WritableStreamDefaultWriter<'_> {
    fn drop(&mut self) {
        // `releaseLock` can throw in hosts like Cloudflare Workers outside a request, and a JS
        // exception escaping a destructor skips Rust cleanup.
        let _ = self.try_release_lock_mut();
    }
}
