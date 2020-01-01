use std::pin::Pin;

use futures::stream::{Stream, TryStreamExt};
use js_sys::Promise;
use wasm_bindgen::prelude::*;

use super::sys;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen_futures::future_to_promise;

#[wasm_bindgen]
pub(crate) struct IntoUnderlyingSource {
    inner: Rc<RefCell<Inner>>,
}

struct Inner {
    stream: Pin<Box<dyn Stream<Item = Result<JsValue, JsValue>>>>,
}

impl IntoUnderlyingSource {
    pub fn new(stream: Box<dyn Stream<Item = Result<JsValue, JsValue>>>) -> Self {
        IntoUnderlyingSource {
            inner: Rc::new(RefCell::new(Inner {
                stream: stream.into(),
            })),
        }
    }
}

#[wasm_bindgen]
impl IntoUnderlyingSource {
    pub fn pull(&self, controller: sys::ReadableStreamDefaultController) -> Promise {
        let inner = self.inner.clone();
        future_to_promise(async move {
            // This mutable borrow can never panic, since the ReadableStream always queues
            // each operation on the underlying source.
            let mut inner = inner.borrow_mut();
            inner.pull(controller).await
        })
    }
}

impl Inner {
    async fn pull(
        &mut self,
        controller: sys::ReadableStreamDefaultController,
    ) -> Result<JsValue, JsValue> {
        match self.stream.as_mut().try_next().await? {
            Some(chunk) => controller.enqueue(&chunk),
            None => controller.close(),
        };
        Ok(JsValue::undefined())
    }
}
