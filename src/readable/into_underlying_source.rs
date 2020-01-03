use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;

use futures::future::{abortable, AbortHandle, TryFutureExt};
use futures::stream::{Stream, TryStreamExt};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use super::sys;

#[wasm_bindgen]
pub(crate) struct IntoUnderlyingSource {
    inner: Rc<RefCell<Inner<Pin<Box<dyn Stream<Item = Result<JsValue, JsValue>>>>>>>,
    pull_handle: Option<AbortHandle>,
}

impl IntoUnderlyingSource {
    pub fn new(stream: Box<dyn Stream<Item = Result<JsValue, JsValue>>>) -> Self {
        IntoUnderlyingSource {
            inner: Rc::new(RefCell::new(Inner::new(stream.into()))),
            pull_handle: None,
        }
    }
}

#[wasm_bindgen]
impl IntoUnderlyingSource {
    pub fn pull(&mut self, controller: sys::ReadableStreamDefaultController) {
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

impl Drop for IntoUnderlyingSource {
    fn drop(&mut self) {
        // Abort the pending pull, if any.
        if let Some(handle) = &mut self.pull_handle {
            handle.abort();
        }
    }
}

struct Inner<S> {
    stream: Option<S>,
}

impl<S> Inner<S>
where
    S: Stream<Item = Result<JsValue, JsValue>> + Unpin,
{
    fn new(stream: S) -> Self {
        Inner {
            stream: Some(stream),
        }
    }

    async fn pull(&mut self, controller: sys::ReadableStreamDefaultController) {
        // The stream should still exist, since pull() will not be called again
        // after the stream has closed or encountered an error.
        let stream = self.stream.as_mut().unwrap_throw();
        match stream.try_next().await {
            Ok(Some(chunk)) => controller.enqueue(&chunk),
            Ok(None) => {
                // The stream has closed, drop it.
                self.stream = None;
                controller.close();
            }
            Err(err) => {
                // The stream encountered an error, drop it.
                self.stream = None;
                controller.error(&err);
            }
        }
    }
}
