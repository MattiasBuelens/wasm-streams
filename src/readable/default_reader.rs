use std::marker::PhantomData;

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::throw_val;

use super::{IntoStream, ReadableStream, sys};

/// A [`ReadableStreamDefaultReader`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamDefaultReader)
/// that can be used to read chunks from a [`ReadableStream`](ReadableStream).
///
/// This is returned by the [`get_reader`](ReadableStream::get_reader) method.
///
/// When the reader is dropped, it automatically [releases its lock](https://streams.spec.whatwg.org/#release-a-lock).
/// This is best-effort: if `releaseLock()` throws, the error is ignored and the stream stays locked.
#[derive(Debug)]
pub struct ReadableStreamDefaultReader<'stream> {
    raw: sys::ReadableStreamDefaultReaderExt,
    _stream: PhantomData<&'stream mut ReadableStream>,
}

impl<'stream> ReadableStreamDefaultReader<'stream> {
    pub(crate) fn new(stream: &mut ReadableStream) -> Result<Self, js_sys::Error> {
        Ok(Self {
            raw: stream
                .as_raw()
                .unchecked_ref::<sys::ReadableStreamExt>()
                .try_get_reader()?,
            _stream: PhantomData,
        })
    }

    /// Acquires a reference to the underlying [JavaScript reader](sys::ReadableStreamDefaultReader).
    #[inline]
    pub fn as_raw(&self) -> &sys::ReadableStreamDefaultReader {
        &self.raw
    }

    #[inline]
    pub(crate) fn as_raw_ext(&self) -> &sys::ReadableStreamDefaultReaderExt {
        &self.raw
    }

    /// Waits for the stream to become closed.
    ///
    /// This returns an error if the stream ever errors, or if the reader's lock is
    /// [released](https://streams.spec.whatwg.org/#release-a-lock) before the stream finishes
    /// closing.
    pub async fn closed(&self) -> Result<(), JsValue> {
        self.as_raw().closed().await.map(|_| ())
    }

    /// [Cancels](https://streams.spec.whatwg.org/#cancel-a-readable-stream) the stream,
    /// signaling a loss of interest in the stream by a consumer.
    ///
    /// Equivalent to [`ReadableStream.cancel`](ReadableStream::cancel).
    pub async fn cancel(&mut self) -> Result<(), JsValue> {
        self.as_raw().cancel().await.map(|_| ())
    }

    /// [Cancels](https://streams.spec.whatwg.org/#cancel-a-readable-stream) the stream,
    /// signaling a loss of interest in the stream by a consumer.
    ///
    /// Equivalent to [`ReadableStream.cancel_with_reason`](ReadableStream::cancel_with_reason).
    pub async fn cancel_with_reason(&mut self, reason: &JsValue) -> Result<(), JsValue> {
        self.as_raw().cancel_with_reason(reason).await.map(|_| ())
    }

    /// Reads the next chunk from the stream's internal queue.
    ///
    /// * If a next `chunk` becomes available, this returns `Ok(Some(chunk))`.
    /// * If the stream closes and no more chunks are available, this returns `Ok(None)`.
    /// * If the stream encounters an `error`, this returns `Err(error)`.
    pub async fn read(&mut self) -> Result<Option<JsValue>, JsValue> {
        let result = self.raw.read().await?;
        if result.get_done().unwrap_or_default() {
            Ok(None)
        } else {
            Ok(Some(result.get_value()))
        }
    }

    /// [Releases](https://streams.spec.whatwg.org/#release-a-lock) this reader's lock on the
    /// corresponding stream.
    ///
    /// [As of January 2022](https://github.com/whatwg/streams/commit/d5f92d9f17306d31ba6b27424d23d58e89bf64a5),
    /// the Streams standard allows the lock to be released even when there are still pending read
    /// requests. Such requests will automatically become rejected, and this function will always
    /// succeed.
    ///
    /// However, if the Streams implementation is not yet up-to-date with this change, then
    /// releasing the lock while there are pending read requests will throw a JavaScript exception.
    /// For a non-throwing variant, use [`try_release_lock`](Self::try_release_lock).
    ///
    /// Some hosts (e.g. Cloudflare Workers outside a request) may also refuse to release the lock,
    /// with the same result.
    #[inline]
    pub fn release_lock(mut self) {
        if let Err(err) = self.try_release_lock_mut() {
            throw_val(err.into())
        }
    }

    /// Try to [release](https://streams.spec.whatwg.org/#release-a-lock) this reader's lock on the
    /// corresponding stream.
    ///
    /// [As of January 2022](https://github.com/whatwg/streams/commit/d5f92d9f17306d31ba6b27424d23d58e89bf64a5),
    /// the Streams standard allows the lock to be released even when there are still pending read
    /// requests. Such requests will automatically become rejected, and this function will always
    /// return `Ok(())`.
    ///
    /// However, if the Streams implementation is not yet up-to-date with this change, then
    /// the lock cannot be released while there are pending read requests. Attempting to do so will
    /// return an error and leave the reader locked to the stream.
    ///
    /// Some hosts (e.g. Cloudflare Workers outside a request) may also refuse to release the lock,
    /// with the same result.
    #[inline]
    pub fn try_release_lock(mut self) -> Result<(), (js_sys::Error, Self)> {
        self.try_release_lock_mut().map_err(|err| (err, self))
    }

    fn try_release_lock_mut(&mut self) -> Result<(), js_sys::Error> {
        self.raw.try_release_lock()
    }

    /// Converts this `ReadableStreamDefaultReader` into a [`Stream`].
    ///
    /// This is similar to [`ReadableStream.into_stream`](ReadableStream::into_stream),
    /// except that after the returned `Stream` is dropped, the original `ReadableStream` is still
    /// usable. This allows reading only a few chunks from the `Stream`, while still allowing
    /// another reader to read the remaining chunks later on.
    ///
    /// [`Stream`]: https://docs.rs/futures/0.3.30/futures/stream/trait.Stream.html
    #[inline]
    pub fn into_stream(self) -> IntoStream<'stream> {
        IntoStream::new(self, false)
    }
}

impl Drop for ReadableStreamDefaultReader<'_> {
    fn drop(&mut self) {
        // `releaseLock` can throw (pending reads on pre-2022 implementations, or Cloudflare
        // Workers outside a request), and a JS exception escaping a destructor skips Rust cleanup.
        let _ = self.try_release_lock_mut();
    }
}
