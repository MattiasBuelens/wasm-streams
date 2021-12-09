use std::marker::PhantomData;

use wasm_bindgen::{throw_val, JsValue};
use wasm_bindgen_futures::JsFuture;

use crate::util::promise_to_void_future;

use super::{sys, IntoStream, ReadableStream};

/// A [`ReadableStreamDefaultReader`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamDefaultReader)
/// that can be used to read chunks from a [`ReadableStream`](ReadableStream).
///
/// This is returned by the [`get_reader`](ReadableStream::get_reader) method.
///
/// When the reader is dropped, it automatically [releases its lock](https://streams.spec.whatwg.org/#release-a-lock).
/// If the reader still has a pending read request at this point (i.e. if a future returned
/// by [`read`](Self::read) is not yet ready), then this will **panic**. You must either `await`
/// all `read` futures, or [`cancel`](Self::cancel) the stream to discard any pending `read` futures.
#[derive(Debug)]
pub struct ReadableStreamDefaultReader<'stream> {
    raw: sys::ReadableStreamDefaultReader,
    _stream: PhantomData<&'stream mut ReadableStream>,
}

impl<'stream> ReadableStreamDefaultReader<'stream> {
    pub(crate) fn new(stream: &mut ReadableStream) -> Result<Self, js_sys::Error> {
        Ok(Self {
            raw: stream.as_raw().get_reader()?,
            _stream: PhantomData,
        })
    }

    /// Acquires a reference to the underlying [JavaScript reader](sys::ReadableStreamDefaultReader).
    #[inline]
    pub fn as_raw(&self) -> &sys::ReadableStreamDefaultReader {
        &self.raw
    }

    /// Waits for the stream to become closed.
    ///
    /// This returns an error if the stream ever errors, or if the reader's lock is
    /// [released](https://streams.spec.whatwg.org/#release-a-lock) before the stream finishes
    /// closing.
    pub async fn closed(&self) -> Result<(), JsValue> {
        promise_to_void_future(self.as_raw().closed()).await
    }

    /// [Cancels](https://streams.spec.whatwg.org/#cancel-a-readable-stream) the stream,
    /// signaling a loss of interest in the stream by a consumer.
    ///
    /// Equivalent to [`ReadableStream.cancel`](ReadableStream::cancel).
    pub async fn cancel(&mut self) -> Result<(), JsValue> {
        promise_to_void_future(self.as_raw().cancel()).await
    }

    /// [Cancels](https://streams.spec.whatwg.org/#cancel-a-readable-stream) the stream,
    /// signaling a loss of interest in the stream by a consumer.
    ///
    /// Equivalent to [`ReadableStream.cancel_with_reason`](ReadableStream::cancel_with_reason).
    pub async fn cancel_with_reason(&mut self, reason: &JsValue) -> Result<(), JsValue> {
        promise_to_void_future(self.as_raw().cancel_with_reason(reason)).await
    }

    /// Reads the next chunk from the stream's internal queue.
    ///
    /// * If a next `chunk` becomes available, this returns `Ok(Some(chunk))`.
    /// * If the stream closes and no more chunks are available, this returns `Ok(None)`.
    /// * If the stream encounters an `error`, this returns `Err(error)`.
    pub async fn read(&mut self) -> Result<Option<JsValue>, JsValue> {
        let promise = self.as_raw().read();
        let js_value = JsFuture::from(promise).await?;
        let result = sys::ReadableStreamDefaultReadResult::from(js_value);
        if result.is_done() {
            Ok(None)
        } else {
            Ok(Some(result.value()))
        }
    }

    /// [Releases](https://streams.spec.whatwg.org/#release-a-lock) this reader's lock on the
    /// corresponding stream.
    ///
    /// **Panics** if the reader still has a pending read request, i.e. if a future returned
    /// by [`read`](Self::read) is not yet ready. For a non-panicking variant,
    /// use [`try_release_lock`](Self::try_release_lock).
    #[inline]
    pub fn release_lock(mut self) {
        self.release_lock_mut()
    }

    fn release_lock_mut(&mut self) {
        self.as_raw()
            .release_lock()
            .unwrap_or_else(|error| throw_val(error.into()))
    }

    /// Try to [release](https://streams.spec.whatwg.org/#release-a-lock) this reader's lock on the
    /// corresponding stream.
    ///
    /// The lock cannot be released while the reader still has a pending read request, i.e.
    /// if a future returned by [`read`](Self::read) is not yet ready. Attempting to do so will
    /// return an error and leave the reader locked to the stream.
    #[inline]
    pub fn try_release_lock(self) -> Result<(), (js_sys::Error, Self)> {
        self.as_raw().release_lock().map_err(|error| (error, self))
    }

    /// Converts this `ReadableStreamDefaultReader` into a [`Stream`].
    ///
    /// This is similar to [`ReadableStream.into_stream`](ReadableStream::into_stream),
    /// except that after the returned `Stream` is dropped, the original `ReadableStream` is still
    /// usable. This allows reading only a few chunks from the `Stream`, while still allowing
    /// another reader to read the remaining chunks later on.
    ///
    /// [`Stream`]: https://docs.rs/futures/0.3.18/futures/stream/trait.Stream.html
    #[inline]
    pub fn into_stream(self) -> IntoStream<'stream> {
        IntoStream::new(self, false)
    }
}

impl Drop for ReadableStreamDefaultReader<'_> {
    fn drop(&mut self) {
        self.release_lock_mut();
    }
}
