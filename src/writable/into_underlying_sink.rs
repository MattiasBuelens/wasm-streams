use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;

use futures::sink::{Sink, SinkExt};
use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

#[wasm_bindgen]
pub(crate) struct IntoUnderlyingSink {
    inner: Rc<RefCell<Inner<Pin<Box<dyn Sink<JsValue, Error = JsValue>>>>>>,
}

impl IntoUnderlyingSink {
    pub fn new(sink: Box<dyn Sink<JsValue, Error = JsValue>>) -> Self {
        IntoUnderlyingSink {
            inner: Rc::new(RefCell::new(Inner::new(sink.into()))),
        }
    }
}

#[wasm_bindgen]
impl IntoUnderlyingSink {
    pub fn write(&mut self, chunk: JsValue) -> Promise {
        let inner = self.inner.clone();
        future_to_promise(async move {
            // This mutable borrow can never panic, since the WritableStream always queues
            // each operation on the underlying sink.
            let mut inner = inner.try_borrow_mut().unwrap_throw();
            inner.write(chunk).await.map(|_| JsValue::undefined())
        })
    }

    pub fn close(self) -> Promise {
        future_to_promise(async move {
            let mut inner = self.inner.try_borrow_mut().unwrap_throw();
            inner.close().await.map(|_| JsValue::undefined())
        })
    }

    pub fn abort(self, reason: JsValue) -> Promise {
        future_to_promise(async move {
            let mut inner = self.inner.try_borrow_mut().unwrap_throw();
            inner.abort(reason).await.map(|_| JsValue::undefined())
        })
    }
}

struct Inner<S> {
    sink: Option<S>,
}

impl<S> Inner<S>
where
    S: Sink<JsValue, Error = JsValue> + Unpin,
{
    fn new(sink: S) -> Self {
        Inner { sink: Some(sink) }
    }

    async fn write(&mut self, chunk: JsValue) -> Result<(), JsValue> {
        // The stream should still exist, since write() will not be called again
        // after the sink has closed, aborted or encountered an error.
        let sink = self.sink.as_mut().unwrap_throw();
        match sink.send(chunk).await {
            Ok(()) => Ok(()),
            Err(err) => {
                // The stream encountered an error, drop it.
                self.sink = None;
                Err(err)
            }
        }
    }

    async fn close(&mut self) -> Result<(), S::Error> {
        let sink = self.sink.as_mut().unwrap_throw();
        let result = sink.close().await;
        self.sink = None;
        result
    }

    async fn abort(&mut self, _reason: JsValue) -> Result<(), S::Error> {
        self.sink = None;
        Ok(())
    }
}
