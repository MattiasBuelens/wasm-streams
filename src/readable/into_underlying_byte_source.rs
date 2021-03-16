use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;

use futures::future::{abortable, AbortHandle, TryFutureExt};
use futures::io::{AsyncRead, AsyncReadExt};
use js_sys::{Error as JsError, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use super::sys;

#[wasm_bindgen]
pub(crate) struct IntoUnderlyingByteSource {
    inner: Rc<RefCell<Inner>>,
    default_buffer_len: usize,
    pull_handle: Option<AbortHandle>,
}

impl IntoUnderlyingByteSource {
    pub fn new(async_read: Box<dyn AsyncRead>, default_buffer_len: usize) -> Self {
        IntoUnderlyingByteSource {
            inner: Rc::new(RefCell::new(Inner::new(async_read))),
            default_buffer_len,
            pull_handle: None,
        }
    }
}

#[wasm_bindgen(inline_js = "export function bytes_literal() { return \"bytes\"; }")]
extern "C" {
    fn bytes_literal() -> JsValue;
}

#[allow(clippy::await_holding_refcell_ref)]
#[wasm_bindgen]
impl IntoUnderlyingByteSource {
    // Chromium has a bug where it only recognizes `new ReadableStream({ type: "bytes" })`,
    // not `new ReadableStream({ type: "by" + "tes" })` or any other non-literal string
    // that equals "bytes". Therefore, we cannot return a Rust `String` here, since
    // that needs to be converted to a JavaScript string at runtime.
    // Instead, we call a function that returns the desired string literal as a `JsValue`
    // and pass that value around. It looks silly, but it works.
    // See https://crbug.com/1187774
    #[wasm_bindgen(getter, js_name = type)]
    pub fn type_(&self) -> JsValue {
        bytes_literal()
    }

    #[wasm_bindgen(getter, js_name = autoAllocateChunkSize)]
    pub fn auto_allocate_chunk_size(&self) -> usize {
        self.default_buffer_len
    }

    pub fn pull(&mut self, controller: sys::ReadableByteStreamController) {
        let inner = self.inner.clone();
        let fut = async move {
            // This mutable borrow can never panic, since the ReadableStream always queues
            // each operation on the underlying source.
            let mut inner = inner.try_borrow_mut().unwrap_throw();
            inner.pull(controller).await;
        };

        // If pull() returns a promise, and the ReadableStream is canceled while the promise
        // from pull() is still pending, it will first await that promise before calling cancel().
        // This would mean that we keep waiting for the next chunk, even though it will be
        // immediately discarded.
        // Therefore, we DO NOT return a promise from pull(), and return nothing instead.
        // This works because when pull() does not return a promise, the ReadableStream will
        // wait until the next enqueue() call before it attempts to call pull() again.
        // See also: https://github.com/whatwg/streams/issues/1014

        // Since we run the future separately, we need to abort it manually when the stream
        // is dropped.
        let (fut, handle) = abortable(fut);
        // Ignore errors from aborting the future.
        let fut = fut.unwrap_or_else(|_| ());

        self.pull_handle = Some(handle);
        spawn_local(fut);
    }

    pub fn cancel(self) {
        // The stream has been canceled, drop everything.
        drop(self);
    }
}

impl Drop for IntoUnderlyingByteSource {
    fn drop(&mut self) {
        // Abort the pending pull, if any.
        if let Some(handle) = &mut self.pull_handle {
            handle.abort();
        }
    }
}

struct Inner {
    async_read: Option<Pin<Box<dyn AsyncRead>>>,
    buffer: Vec<u8>,
}

impl Inner {
    fn new(async_read: Box<dyn AsyncRead>) -> Self {
        Inner {
            async_read: Some(async_read.into()),
            buffer: Vec::new(),
        }
    }

    async fn pull(&mut self, controller: sys::ReadableByteStreamController) {
        // The AsyncRead should still exist, since pull() will not be called again
        // after the stream has closed or encountered an error.
        let async_read = self.async_read.as_mut().unwrap_throw();
        // We set autoAllocateChunkSize, so there should always be a BYOB request.
        let request = controller.byob_request().unwrap_throw();
        // Resize the buffer to fit the BYOB request.
        let request_view = request.view().unwrap_throw();
        let request_len = request_view.byte_length() as usize;
        if self.buffer.len() < request_len {
            self.buffer.resize(request_len, 0);
        }
        match async_read.read(&mut self.buffer[0..request_len]).await {
            Ok(0) => {
                // The stream has closed, drop it.
                self.discard();
                controller.close();
                request.respond(0);
            }
            Ok(bytes_read) => {
                // Copy read bytes from buffer to BYOB request view
                let dest = Uint8Array::new_with_byte_offset_and_length(
                    &request_view.buffer(),
                    request_view.byte_offset(),
                    request_view.byte_length(),
                );
                let bytes = &self.buffer[0..bytes_read];
                unsafe {
                    // This is safe because `set()` copies from its argument
                    dest.set(&Uint8Array::view(bytes), 0);
                }
                // Respond to BYOB request
                debug_assert!(bytes_read <= u32::MAX as usize);
                request.respond(bytes_read as u32);
            }
            Err(err) => {
                // The stream encountered an error, drop it.
                self.discard();
                controller.error(&JsError::new(&err.to_string()));
            }
        }
    }

    #[inline]
    fn discard(&mut self) {
        self.async_read = None;
        self.buffer = Vec::new();
    }
}
