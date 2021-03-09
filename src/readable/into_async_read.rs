use core::pin::Pin;

use futures::future::Future;
use futures::io::{AsyncRead, Error, ErrorKind};
use futures::ready;
use futures::task::{Context, Poll};
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use super::sys::ReadableStreamBYOBReadResult;
use super::ReadableStreamBYOBReader;

/// An [`AsyncRead`](AsyncRead) for the [`into_async_read`](super::ReadableStream::into_async_read) method.
///
/// This stream holds a reader, and therefore locks the [`ReadableStream`](super::ReadableStream).
/// When this stream is dropped, it also drops its reader which in turn
/// [releases its lock](https://streams.spec.whatwg.org/#release-a-lock).
#[must_use = "streams do nothing unless polled"]
#[derive(Debug)]
pub struct IntoAsyncRead<'reader> {
    reader: Option<ReadableStreamBYOBReader<'reader>>,
    buffer: Option<Uint8Array>,
    fut: Option<JsFuture>,
}

impl<'reader> IntoAsyncRead<'reader> {
    #[inline]
    pub(super) fn new(reader: ReadableStreamBYOBReader) -> IntoAsyncRead {
        IntoAsyncRead {
            reader: Some(reader),
            buffer: None,
            fut: None,
        }
    }

    #[inline]
    fn discard_reader(mut self: Pin<&mut Self>) {
        self.reader = None;
        self.buffer = None;
    }
}

impl<'reader> AsyncRead for IntoAsyncRead<'reader> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Error>> {
        if self.fut.is_none() {
            // No pending read, start reading the next bytes
            let buf_len = buf.len() as u32;
            let buffer = match self.as_mut().buffer.take() {
                // Re-use the internal buffer if it is large enough,
                // otherwise allocate a new one
                Some(buffer) if buffer.byte_length() >= buf_len => buffer,
                _ => Uint8Array::new_with_length(buf_len),
            };
            // Limit to output buffer size
            let buffer = buffer.subarray(0, buf_len);
            match self.reader.as_ref() {
                Some(reader) => {
                    // Read into internal buffer and store its future
                    let fut = JsFuture::from(reader.as_raw().read(&buffer));
                    self.as_mut().fut = Some(fut);
                }
                None => {
                    // Reader was already dropped
                    return Poll::Ready(Ok(0));
                }
            }
        }

        // Poll the future for the pending read
        let js_result = ready!(Pin::new(self.as_mut().fut.as_mut().unwrap_throw()).poll(cx));
        self.as_mut().fut = None;

        // Read completed
        Poll::Ready(match js_result {
            Ok(js_value) => {
                let result = ReadableStreamBYOBReadResult::from(js_value);
                let filled_view = result.value();
                if result.is_done() {
                    // End of stream
                    self.discard_reader();
                    Ok(0)
                } else {
                    // Copy bytes to output buffer
                    debug_assert!(filled_view.byte_length() <= buf.len() as u32);
                    filled_view.copy_to(buf);
                    // Re-construct internal buffer with the new ArrayBuffer
                    self.as_mut().buffer = Some(Uint8Array::new(&filled_view.buffer()));
                    Ok(filled_view.byte_length() as usize)
                }
            }
            Err(js_value) => {
                // Error
                self.discard_reader();
                Err(Error::new(ErrorKind::Other, "TODO js_value to error"))
            }
        })
    }
}
