use core::pin::Pin;

use futures::future::Future;
use futures::ready;
use futures::sink::Sink;
use futures::task::{Context, Poll};
use pin_utils::{unsafe_pinned, unsafe_unpinned};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use super::WritableStreamDefaultWriter;

pub struct IntoSink<'writer> {
    writer: Option<WritableStreamDefaultWriter<'writer>>,
    ready_fut: Option<JsFuture>,
    write_fut: Option<JsFuture>,
    close_fut: Option<JsFuture>,
}

impl<'writer> IntoSink<'writer> {
    unsafe_unpinned!(writer: Option<WritableStreamDefaultWriter<'writer>>);
    unsafe_pinned!(ready_fut: Option<JsFuture>);
    unsafe_pinned!(write_fut: Option<JsFuture>);
    unsafe_pinned!(close_fut: Option<JsFuture>);

    #[inline]
    pub(super) fn new(writer: WritableStreamDefaultWriter) -> IntoSink {
        IntoSink {
            writer: Some(writer),
            ready_fut: None,
            write_fut: None,
            close_fut: None,
        }
    }
}

impl<'writer> Sink<JsValue> for IntoSink<'writer> {
    type Error = JsValue;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.as_ref().ready_fut.is_none() {
            // No pending ready future, start reading the next chunk
            match self.as_ref().writer.as_ref() {
                Some(writer) => {
                    // Create future for ready promise
                    let fut = JsFuture::from(writer.as_raw().ready());
                    self.as_mut().ready_fut().set(Some(fut));
                }
                None => {
                    // Writer was already dropped
                    // TODO Return error?
                    return Poll::Ready(Ok(()));
                }
            }
        }

        // Poll the ready future
        let js_result = ready!(self.as_mut().ready_fut().as_pin_mut().unwrap().poll(cx));
        self.as_mut().ready_fut().set(None);

        // Ready future completed
        Poll::Ready(match js_result {
            Ok(js_value) => {
                debug_assert!(js_value.is_undefined());
                Ok(())
            }
            Err(js_value) => {
                // Error, drop writer
                *self.as_mut().writer() = None;
                Err(js_value)
            }
        })
    }

    fn start_send(mut self: Pin<&mut Self>, item: JsValue) -> Result<(), Self::Error> {
        match self.as_ref().writer.as_ref() {
            Some(writer) => {
                let fut = JsFuture::from(writer.as_raw().write(item));
                // Set or replace the pending write future
                self.as_mut().write_fut().set(Some(fut));
                Ok(())
            }
            None => {
                // TODO Return error?
                Ok(())
            }
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // If we're not writing, then there's nothing to flush
        if self.as_ref().write_fut.is_none() {
            return Poll::Ready(Ok(()));
        }

        // Poll the write future
        let js_result = ready!(self.as_mut().write_fut().as_pin_mut().unwrap().poll(cx));
        self.as_mut().write_fut().set(None);

        // Write future completed
        Poll::Ready(match js_result {
            Ok(js_value) => {
                debug_assert!(js_value.is_undefined());
                Ok(())
            }
            Err(js_value) => {
                // Error, drop writer
                *self.as_mut().writer() = None;
                Err(js_value)
            }
        })
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.as_ref().close_fut.is_none() {
            // No pending close future, start closing the stream
            match self.as_ref().writer.as_ref() {
                Some(writer) => {
                    // Create future for close promise
                    let fut = JsFuture::from(writer.as_raw().close());
                    self.as_mut().close_fut().set(Some(fut));
                }
                None => {
                    // Writer was already dropped
                    // TODO Return error?
                    return Poll::Ready(Ok(()));
                }
            }
        }

        // Poll the close future
        let js_result = ready!(self.as_mut().close_fut().as_pin_mut().unwrap().poll(cx));
        self.as_mut().close_fut().set(None);

        // Close future completed
        *self.as_mut().writer() = None;
        Poll::Ready(match js_result {
            Ok(js_value) => {
                debug_assert!(js_value.is_undefined());
                Ok(())
            }
            Err(js_value) => {
                Err(js_value)
            }
        })
    }
}