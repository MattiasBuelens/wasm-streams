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
    stream: Rc<RefCell<Pin<Box<dyn Stream<Item = Result<JsValue, JsValue>>>>>>,
}

impl IntoUnderlyingSource {
    pub fn new(stream: Box<dyn Stream<Item = Result<JsValue, JsValue>>>) -> Self {
        IntoUnderlyingSource {
            stream: Rc::new(RefCell::new(stream.into())),
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
            let mut stream = stream.borrow_mut();
            match stream.as_mut().try_next().await? {
                Some(chunk) => controller.enqueue(&chunk),
                None => controller.close(),
            };
            Ok(JsValue::undefined())
        })
    }
}
