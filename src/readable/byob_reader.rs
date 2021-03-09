use std::marker::PhantomData;

use js_sys::Uint8Array;
use wasm_bindgen::{throw_val, JsValue};
use wasm_bindgen_futures::JsFuture;

use crate::util::promise_to_void_future;

use super::{sys, IntoAsyncRead, ReadableStream};

/// A [`ReadableStreamBYOBReader`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamBYOBReader)
/// that can be used to read chunks from a [`ReadableStream`](ReadableStream).
///
/// This is returned by the [`get_byob_reader`](ReadableStream::get_byob_reader) method.
///
/// When the reader is dropped, it automatically [releases its lock](https://streams.spec.whatwg.org/#release-a-lock).
#[derive(Debug)]
pub struct ReadableStreamBYOBReader<'stream> {
    raw: sys::ReadableStreamBYOBReader,
    _stream: PhantomData<&'stream mut ReadableStream>,
}

impl<'stream> ReadableStreamBYOBReader<'stream> {
    pub(crate) fn new(stream: &mut ReadableStream) -> Result<Self, js_sys::Error> {
        Ok(Self {
            raw: stream.as_raw().get_reader_with_options(
                sys::ReadableStreamGetReaderOptions::new(sys::ReadableStreamReaderMode::BYOB),
            )?,
            _stream: PhantomData,
        })
    }

    /// Acquires a reference to the underlying [JavaScript reader](sys::ReadableStreamBYOBReader).
    #[inline]
    pub fn as_raw(&self) -> &sys::ReadableStreamBYOBReader {
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

    /// Reads the next chunk from the stream's internal queue into `dst`,
    /// and returns the number of bytes read.
    ///
    /// * If some bytes were read into `dst`, this returns `Ok(bytes_read)`.
    /// * If the stream closes and no more bytes are available, this returns `Ok(0)`.
    /// * If the stream encounters an `error`, this returns `Err(error)`.
    ///
    /// This always allocated a new temporary `Uint8Array` with the same size as `dst` to hold
    /// the result before copying to `dst`. We cannot pass a view on the backing WebAssembly memory
    /// directly, because:
    /// * `reader.read(view)` needs to transfer `view.buffer`, but `WebAssembly.Memory` buffers
    ///    are non-transferable
    /// * `view.buffer` can be invalidated if the WebAssembly memory grows while `read(view)`
    ///    is still in progress.
    ///
    /// Therefore, it is necessary to use a separate buffer living in the JavaScript heap.
    /// To avoid repeated allocations for repeated reads,
    /// use [`read_with_buffer`](Self::read_with_buffer).
    pub async fn read(&mut self, dst: &mut [u8]) -> Result<usize, JsValue> {
        let buffer = Uint8Array::new_with_length(dst.len() as u32); // TODO Clamp to u32::MAX
        let (bytes_read, _) = self.read_with_buffer(dst, buffer).await?;
        Ok(bytes_read)
    }

    /// Reads the next chunk from the stream's internal queue into `dst`,
    /// and returns the number of bytes read.
    ///
    /// The given `buffer` is used to store the bytes before they are copied to `dst`.
    /// This buffer is returned back together with the result, so it can be re-used for subsequent
    /// reads without extra allocations. Note that the underlying `ArrayBuffer` is transferred
    /// in the process, so any other views on the original buffer will become unusable.
    ///
    /// * If some bytes were read into `dst`, this returns `Ok((bytes_read, buffer))`.
    /// * If the stream closes and no more bytes are available, this returns `Ok((0, buffer))`.
    /// * If the stream encounters an `error`, this returns `Err(error)`.
    pub async fn read_with_buffer(
        &mut self,
        dst: &mut [u8],
        buffer: Uint8Array,
    ) -> Result<(usize, Uint8Array), JsValue> {
        // Save the original buffer's byte offset and length.
        let buffer_offset = buffer.byte_offset();
        let byte_length = buffer.byte_length();
        // Limit view to destination slice's length.
        let mut view = buffer.subarray(0, dst.len() as u32); // TODO Clamp to u32::MAX
                                                             // Read into view. This transfers `buffer.buffer()`.
        let promise = self.as_raw().read(&mut view);
        let js_value = JsFuture::from(promise).await?;
        let result = sys::ReadableStreamBYOBReadResult::from(js_value);
        let filled_view = result.value();
        debug_assert!(filled_view.byte_length() <= dst.len() as u32);
        // Re-construct the original Uint8Array with the new ArrayBuffer.
        let new_buffer = Uint8Array::new_with_byte_offset_and_length(
            &filled_view.buffer(),
            buffer_offset,
            byte_length,
        );
        if result.is_done() {
            debug_assert_eq!(filled_view.byte_length(), 0);
            Ok((0, new_buffer))
        } else {
            filled_view.copy_to(dst);
            Ok((filled_view.byte_length() as usize, new_buffer))
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

    /// Converts this `ReadableStreamBYOBReader` into an [`AsyncRead`](futures::io::AsyncRead).
    ///
    /// This is similar to [`ReadableStream.into_async_read`](ReadableStream::into_async_read),
    /// except that after the returned `AsyncRead` is dropped, the original `ReadableStream` is
    /// still usable. This allows reading only a few bytes from the `AsyncRead`, while still
    /// allowing another reader to read the remaining bytes later on.
    #[inline]
    pub fn into_async_read(self) -> IntoAsyncRead<'stream> {
        IntoAsyncRead::new(self)
    }
}

impl Drop for ReadableStreamBYOBReader<'_> {
    fn drop(&mut self) {
        self.release_lock_mut();
    }
}
