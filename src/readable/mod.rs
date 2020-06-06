//! Bindings and conversions for
//! [readable streams](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream).
use std::future::Future;
use std::marker::PhantomData;

use futures::stream::Stream;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

pub use into_stream::IntoStream;
use into_underlying_source::IntoUnderlyingSource;

use super::queuing_strategy::QueuingStrategy;

mod into_stream;
mod into_underlying_source;
pub mod sys;

/// A [`ReadableStream`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream).
///
/// `ReadableStream`s can be created from a [raw JavaScript stream](sys::ReadableStream) with
/// [`from_raw`](Self::from_raw), or from a Rust [`Stream`](Stream)
/// with [`from_stream`](Self::from_stream).
///
/// They can be converted into a [raw JavaScript stream](sys::ReadableStream) with
/// [`into_raw`](Self::into_raw), or into a Rust [`Stream`](Stream)
/// with [`into_stream`](Self::into_stream).
#[derive(Debug)]
pub struct ReadableStream {
    raw: sys::ReadableStream,
}

impl ReadableStream {
    /// Creates a new `ReadableStream` from a [JavaScript stream](sys::ReadableStream).
    #[inline]
    pub fn from_raw(raw: sys::ReadableStream) -> Self {
        Self { raw }
    }

    /// Creates a new `ReadableStream` from a [`Stream`](Stream).
    ///
    /// Items and errors must be represented as raw [`JsValue`](JsValue)s.
    /// Use [`map`](futures::StreamExt::map), [`map_ok`](futures::TryStreamExt::map_ok) and/or
    /// [`map_err`](futures::TryStreamExt::map_err) to convert a stream's items to a `JsValue`
    /// before passing it to this function.
    pub fn from_stream<St>(stream: St) -> Self
    where
        St: Stream<Item = Result<JsValue, JsValue>> + 'static,
    {
        let source = IntoUnderlyingSource::new(Box::new(stream));
        // Set HWM to 0 to prevent the JS ReadableStream from buffering chunks in its queue,
        // since the original Rust stream is better suited to handle that.
        let strategy = QueuingStrategy::new(0.0);
        let raw = sys::ReadableStream::new_with_source(source, strategy);
        Self { raw }
    }

    /// Acquires a reference to the underlying [JavaScript stream](sys::ReadableStream).
    #[inline]
    pub fn as_raw(&self) -> &sys::ReadableStream {
        &self.raw
    }

    /// Consumes this `ReadableStream`, returning the underlying [JavaScript stream](sys::ReadableStream).
    #[inline]
    pub fn into_raw(self) -> sys::ReadableStream {
        self.raw
    }

    /// Returns `true` if the stream is [locked to a reader](https://streams.spec.whatwg.org/#lock).
    pub fn is_locked(&self) -> bool {
        self.raw.is_locked()
    }

    /// [Cancels](https://streams.spec.whatwg.org/#cancel-a-readable-stream) the stream,
    /// signaling a loss of interest in the stream by a consumer.
    ///
    /// If the stream is currently locked to a reader, then this returns an error.
    pub fn cancel<'a>(&'a mut self) -> impl Future<Output = Result<(), JsValue>> + 'a {
        let promise = self.raw.cancel();
        async {
            let js_value = JsFuture::from(promise).await?;
            debug_assert!(js_value.is_undefined());
            Ok(())
        }
    }

    /// [Cancels](https://streams.spec.whatwg.org/#cancel-a-readable-stream) the stream,
    /// signaling a loss of interest in the stream by a consumer.
    ///
    /// The supplied `reason` will be given to the underlying source, which may or may not use it.
    ///
    /// If the stream is currently locked to a reader, then this returns an error.
    pub fn cancel_with_reason<'a>(
        &'a mut self,
        reason: &JsValue,
    ) -> impl Future<Output = Result<(), JsValue>> + 'a {
        let promise = self.raw.cancel_with_reason(reason);
        async {
            let js_value = JsFuture::from(promise).await?;
            debug_assert!(js_value.is_undefined());
            Ok(())
        }
    }

    /// Creates a [default reader](ReadableStreamDefaultReader) and
    /// [locks](https://streams.spec.whatwg.org/#lock) the stream to the new reader.
    ///
    /// While the stream is locked, no other reader can be acquired until this one is released.
    ///
    /// If the stream is already locked to a reader, then this returns an error.
    pub fn get_reader(&mut self) -> Result<ReadableStreamDefaultReader, js_sys::Error> {
        Ok(ReadableStreamDefaultReader {
            raw: Some(self.raw.get_reader()?),
            _stream: PhantomData,
        })
    }

    /// Converts this `ReadableStream` into a [`Stream`](Stream).
    ///
    /// Items and errors are represented by their raw [`JsValue`](JsValue).
    /// Use [`map`](futures::StreamExt::map), [`map_ok`](futures::TryStreamExt::map_ok) and/or
    /// [`map_err`](futures::TryStreamExt::map_err) on the returned stream to convert them to a more
    /// appropriate type.
    ///
    /// If the stream is already locked to a reader, then this returns an error
    /// along with the original `ReadableStream`.
    pub fn into_stream(self) -> Result<IntoStream<'static>, (js_sys::Error, Self)> {
        let raw_reader = match self.raw.get_reader() {
            Ok(raw_reader) => raw_reader,
            Err(err) => return Err((err, self)),
        };
        let reader = ReadableStreamDefaultReader {
            raw: Some(raw_reader),
            _stream: PhantomData,
        };
        Ok(reader.into_stream())
    }
}

impl<St> From<St> for ReadableStream
where
    St: Stream<Item = Result<JsValue, JsValue>> + 'static,
{
    /// Equivalent to [`from_stream`](Self::from_stream).
    #[inline]
    fn from(stream: St) -> Self {
        Self::from_stream(stream)
    }
}

/// A [`ReadableStreamDefaultReader`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamDefaultReader)
/// that can be used to read chunks from a [`ReadableStream`](ReadableStream).
///
/// This is returned by the [`get_reader`](ReadableStream::get_reader) method.
///
/// When the reader is dropped, it automatically [releases its lock](https://streams.spec.whatwg.org/#release-a-lock).
#[derive(Debug)]
pub struct ReadableStreamDefaultReader<'stream> {
    raw: Option<sys::ReadableStreamDefaultReader>,
    _stream: PhantomData<&'stream mut ReadableStream>,
}

impl<'stream> ReadableStreamDefaultReader<'stream> {
    /// Acquires a reference to the underlying [JavaScript reader](sys::ReadableStreamDefaultReader).
    #[inline]
    pub fn as_raw(&self) -> &sys::ReadableStreamDefaultReader {
        self.raw.as_ref().unwrap_throw()
    }

    /// Waits for the stream to become closed.
    ///
    /// This returns an error if the stream ever errors, or if the reader's lock is
    /// [released](https://streams.spec.whatwg.org/#release-a-lock) before the stream finishes
    /// closing.
    pub async fn closed(&self) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.as_raw().closed()).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    /// [Cancels](https://streams.spec.whatwg.org/#cancel-a-readable-stream) the stream,
    /// signaling a loss of interest in the stream by a consumer.
    ///
    /// Equivalent to [`ReadableStream.cancel`](ReadableStream::cancel).
    pub fn cancel<'a>(&'a mut self) -> impl Future<Output = Result<(), JsValue>> + 'a {
        let promise = self.as_raw().cancel();
        async {
            let js_value = JsFuture::from(promise).await?;
            debug_assert!(js_value.is_undefined());
            Ok(())
        }
    }

    /// [Cancels](https://streams.spec.whatwg.org/#cancel-a-readable-stream) the stream,
    /// signaling a loss of interest in the stream by a consumer.
    ///
    /// Equivalent to [`ReadableStream.cancel_with_reason`](ReadableStream::cancel_with_reason).
    pub fn cancel_with_reason<'a>(
        &'a mut self,
        reason: &JsValue,
    ) -> impl Future<Output = Result<(), JsValue>> + 'a {
        let promise = self.as_raw().cancel_with_reason(reason);
        async {
            let js_value = JsFuture::from(promise).await?;
            debug_assert!(js_value.is_undefined());
            Ok(())
        }
    }

    /// Reads the next chunk from the stream's internal queue.
    ///
    /// * If a next `chunk` becomes available, this returns `Ok(Some(chunk))`.
    /// * If the stream closes and no more chunks are available, this returns `Ok(None)`.
    /// * If the stream encounters an `error`, this returns `Err(error)`.
    pub fn read<'a>(&'a mut self) -> impl Future<Output = Result<Option<JsValue>, JsValue>> + 'a {
        let promise = self.as_raw().read();
        async {
            let js_value = JsFuture::from(promise).await?;
            let result = sys::ReadableStreamReadResult::from(js_value);
            if result.is_done() {
                Ok(None)
            } else {
                Ok(Some(result.value()))
            }
        }
    }

    /// [Releases](https://streams.spec.whatwg.org/#release-a-lock) this reader's lock on the
    /// corresponding stream.
    ///
    /// The lock cannot be released while the reader still has a pending read request, i.e.
    /// if a future returned by [`read`](Self::read) is not yet ready. Attempting to do so will
    /// return an error and leave the reader locked to the stream.
    pub fn release_lock(&mut self) -> Result<(), js_sys::Error> {
        if let Some(raw) = self.raw.as_ref() {
            raw.release_lock()?;
            self.raw.take();
        }
        Ok(())
    }

    /// Converts this `ReadableStreamDefaultReader` into a [`Stream`](Stream).
    ///
    /// This is similar to [`ReadableStream.into_stream`](ReadableStream::into_stream),
    /// except that after the returned `Stream` is dropped, the original `ReadableStream` is still
    /// usable. This allows reading only a few chunks from the `Stream`, while still allowing
    /// another reader to read the remaining chunks later on.
    pub fn into_stream(self) -> IntoStream<'stream> {
        IntoStream::new(self)
    }
}

impl Drop for ReadableStreamDefaultReader<'_> {
    fn drop(&mut self) {
        // TODO Error handling?
        self.release_lock().unwrap_throw();
    }
}
