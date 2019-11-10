use std::cell::RefCell;
use std::rc::Rc;

use js_sys::{Object, Promise};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::future_to_promise;

use async_trait::async_trait;
pub use sys::TransformStreamDefaultController;

use crate::readable::ReadableStream;
use crate::writable::WritableStream;

pub mod sys;

pub struct TransformStream {
    inner: sys::TransformStream,
    _transformer: Option<JsTransformer>,
}

impl TransformStream {
    pub fn new(source: Box<dyn Transformer + 'static>) -> TransformStream {
        let transformer = JsTransformer::new(source);
        let inner = sys::TransformStream::new_with_transformer(transformer.as_raw());
        TransformStream {
            inner,
            _transformer: Some(transformer),
        }
    }

    #[inline]
    pub fn as_raw(&self) -> &sys::TransformStream {
        &self.inner
    }

    pub fn readable(&self) -> ReadableStream {
        ReadableStream::from(self.inner.readable())
    }

    pub fn writable(&self) -> WritableStream {
        WritableStream::from(self.inner.writable())
    }

    pub fn forget(self) -> sys::TransformStream {
        if let Some(transformer) = self._transformer {
            transformer.forget();
        }
        self.inner
    }
}

impl From<sys::TransformStream> for TransformStream {
    fn from(raw: sys::TransformStream) -> TransformStream {
        TransformStream {
            inner: raw,
            _transformer: None,
        }
    }
}

#[async_trait(? Send)]
pub trait Transformer {
    async fn start(&mut self, controller: &TransformStreamDefaultController) -> Result<(), JsValue> {
        let _ = controller;
        Ok(())
    }

    async fn transform(&mut self, chunk: JsValue, controller: &TransformStreamDefaultController) -> Result<(), JsValue> {
        controller.enqueue(&chunk);
        Ok(())
    }

    async fn flush(&mut self, controller: &TransformStreamDefaultController) -> Result<(), JsValue> {
        let _ = controller;
        Ok(())
    }
}

struct JsTransformer {
    inner: sys::Transformer,
    start_closure: Closure<dyn FnMut(TransformStreamDefaultController) -> Promise>,
    transform_closure: Closure<dyn FnMut(JsValue, TransformStreamDefaultController) -> Promise>,
    flush_closure: Closure<dyn FnMut(TransformStreamDefaultController) -> Promise>,
}

impl JsTransformer {
    pub fn new(transformer: Box<dyn Transformer + 'static>) -> JsTransformer {
        let transformer = Rc::new(RefCell::new(transformer));

        let start_closure = {
            let transformer = transformer.clone();
            Closure::wrap(Box::new(move |controller: TransformStreamDefaultController| {
                let transformer = transformer.clone();
                future_to_promise(async move {
                    // This mutable borrow can never panic, since the TransformStream always
                    // queues each operation on the transformer.
                    let mut transformer = transformer.borrow_mut();
                    transformer.start(&controller).await?;
                    Ok(JsValue::undefined())
                })
            }) as Box<dyn FnMut(TransformStreamDefaultController) -> Promise>)
        };
        let transform_closure = {
            let transformer = transformer.clone();
            Closure::wrap(Box::new(move |chunk: JsValue, controller: TransformStreamDefaultController| {
                let transformer = transformer.clone();
                future_to_promise(async move {
                    let mut transformer = transformer.borrow_mut();
                    transformer.transform(chunk, &controller).await?;
                    Ok(JsValue::undefined())
                })
            }) as Box<dyn FnMut(JsValue, TransformStreamDefaultController) -> Promise>)
        };
        let flush_closure = {
            let transformer = transformer.clone();
            Closure::wrap(Box::new(move |controller: TransformStreamDefaultController| {
                let transformer = transformer.clone();
                future_to_promise(async move {
                    let mut transformer = transformer.borrow_mut();
                    transformer.flush(&controller).await?;
                    Ok(JsValue::undefined())
                })
            }) as Box<dyn FnMut(TransformStreamDefaultController) -> Promise>)
        };

        let inner = sys::Transformer::from(JsValue::from(Object::new()));
        inner.set_start(&start_closure);
        inner.set_transform(&transform_closure);
        inner.set_flush(&flush_closure);

        JsTransformer {
            inner,
            start_closure,
            transform_closure,
            flush_closure,
        }
    }

    #[inline]
    pub fn as_raw(&self) -> &sys::Transformer {
        &self.inner
    }

    pub fn forget(self) -> sys::Transformer {
        self.start_closure.forget();
        self.transform_closure.forget();
        self.flush_closure.forget();
        self.inner
    }
}