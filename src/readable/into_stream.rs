use core::pin::Pin;

use futures::future::FutureExt;
use futures::ready;
use futures::stream::{FusedStream, Stream};
use futures::task::{Context, Poll};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use super::sys::ReadableStreamReadResult;
use super::ReadableStreamDefaultReader;

/// A [`Stream`](futures::Stream) for the [`into_stream`](super::ReadableStream::into_stream) method.
///
/// This stream holds a reader, and therefore locks the [`ReadableStream`](super::ReadableStream).
/// When this stream is dropped, it also drops its reader which in turn
/// [releases its lock](https://streams.spec.whatwg.org/#release-a-lock).
#[must_use = "streams do nothing unless polled"]
#[derive(Debug)]
pub struct IntoStream<'reader> {
    reader: Option<ReadableStreamDefaultReader<'reader>>,
    fut: Option<JsFuture>,
}

impl<'reader> IntoStream<'reader> {
    #[inline]
    pub(super) fn new(reader: ReadableStreamDefaultReader) -> IntoStream {
        IntoStream {
            reader: Some(reader),
            fut: None,
        }
    }

    /// [Cancels](https://streams.spec.whatwg.org/#cancel-a-readable-stream) the stream,
    /// signaling a loss of interest in the stream by a consumer.
    pub async fn cancel(mut self) -> Result<(), JsValue> {
        if let Some(mut reader) = self.reader.take() {
            reader.cancel().await?
        }
        Ok(())
    }

    /// [Cancels](https://streams.spec.whatwg.org/#cancel-a-readable-stream) the stream,
    /// signaling a loss of interest in the stream by a consumer.
    pub async fn cancel_with_reason(mut self, reason: &JsValue) -> Result<(), JsValue> {
        if let Some(mut reader) = self.reader.take() {
            reader.cancel_with_reason(reason).await?
        }
        Ok(())
    }
}

impl FusedStream for IntoStream<'_> {
    fn is_terminated(&self) -> bool {
        self.reader.is_none() && self.fut.is_none()
    }
}

impl<'reader> Stream for IntoStream<'reader> {
    type Item = Result<JsValue, JsValue>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.fut.is_none() {
            // No pending read, start reading the next chunk
            match &self.reader {
                Some(reader) => {
                    // Read a chunk and store its future
                    let fut = JsFuture::from(reader.as_raw().read());
                    self.fut = Some(fut);
                }
                None => {
                    // Reader was already dropped
                    return Poll::Ready(None);
                }
            }
        }

        // Poll the future for the pending read
        let js_result = ready!(self.as_mut().fut.as_mut().unwrap_throw().poll_unpin(cx));
        self.fut = None;

        // Read completed
        Poll::Ready(match js_result {
            Ok(js_value) => {
                let result = ReadableStreamReadResult::from(js_value);
                if result.is_done() {
                    // End of stream, drop reader
                    self.reader = None;
                    None
                } else {
                    Some(Ok(result.value()))
                }
            }
            Err(js_value) => {
                // Error, drop reader
                self.reader = None;
                Some(Err(js_value))
            }
        })
    }
}
