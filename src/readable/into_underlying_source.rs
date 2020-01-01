use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;

use futures::stream::{Stream, TryStreamExt};
use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use super::sys;

#[wasm_bindgen]
pub(crate) struct IntoUnderlyingSource {
    stream: Rc<RefCell<Option<Pin<Box<dyn Stream<Item = Result<JsValue, JsValue>>>>>>>,
}

impl IntoUnderlyingSource {
    pub fn new(stream: Box<dyn Stream<Item = Result<JsValue, JsValue>>>) -> Self {
        IntoUnderlyingSource {
            stream: Rc::new(RefCell::new(Some(stream.into()))),
        }
    }
}

#[wasm_bindgen]
impl IntoUnderlyingSource {
    pub fn pull(&self, controller: sys::ReadableStreamDefaultController) -> Promise {
        let stream = self.stream.clone();
        future_to_promise(async move {
            // This mutable borrow can never panic, since the ReadableStream always queues
            // each operation on the underlying source.
            let mut maybe_stream = stream.borrow_mut();
            // The stream should still exist, since pull() will not be called again
            // after the stream has closed or encountered an error.
            let stream = maybe_stream.as_mut().unwrap_throw();
            match stream.try_next().await {
                Ok(Some(chunk)) => controller.enqueue(&chunk),
                Ok(None) => {
                    // The stream has closed, drop it.
                    *maybe_stream = None;
                    controller.close();
                }
                Err(err) => {
                    // The stream encountered an error, drop it.
                    *maybe_stream = None;
                    controller.error(&err);
                }
            }
            Ok(JsValue::undefined())
        })
    }

    pub fn cancel(self) {
        // The stream has been canceled, drop everything.
        drop(self);
    }
}
