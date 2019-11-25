use std::cell::RefCell;
use std::rc::Rc;

use js_sys::{Object, Promise};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use async_trait::async_trait;

use crate::readable::ReadableStream;
use crate::writable::WritableStream;

pub mod sys;

pub struct TransformStream {
    raw: sys::TransformStream,
    _transformer: Option<JsTransformer>,
}

impl TransformStream {
    pub fn new(source: Box<dyn Transformer + 'static>) -> TransformStream {
        let transformer = JsTransformer::new(source);
        let raw = sys::TransformStream::new_with_transformer(transformer.as_raw());
        TransformStream {
            raw,
            _transformer: Some(transformer),
        }
    }

    #[inline]
    pub fn as_raw(&self) -> &sys::TransformStream {
        &self.raw
    }

    pub fn readable(&self) -> ReadableStream {
        ReadableStream::from(self.raw.readable())
    }

    pub fn writable(&self) -> WritableStream {
        WritableStream::from(self.raw.writable())
    }

    pub fn forget(self) -> sys::TransformStream {
        if let Some(transformer) = self._transformer {
            transformer.forget();
        }
        self.raw
    }
}

impl From<sys::TransformStream> for TransformStream {
    fn from(raw: sys::TransformStream) -> TransformStream {
        TransformStream {
            raw,
            _transformer: None,
        }
    }
}

pub struct TransformStreamDefaultController {
    raw: sys::TransformStreamDefaultController
}

impl TransformStreamDefaultController {
    #[inline]
    pub fn as_raw(&self) -> &sys::TransformStreamDefaultController {
        &self.raw
    }

    pub fn desired_size(&self) -> Option<f64> {
        self.raw.desired_size()
    }

    pub fn enqueue(&self, chunk: &JsValue) {
        self.raw.enqueue(chunk)
    }

    pub fn error(&self, error: &JsValue) {
        self.raw.error(error)
    }

    pub fn terminate(&self) {
        self.raw.terminate()
    }
}

impl From<sys::TransformStreamDefaultController> for TransformStreamDefaultController {
    fn from(raw: sys::TransformStreamDefaultController) -> TransformStreamDefaultController {
        TransformStreamDefaultController {
            raw
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
    raw: sys::Transformer,
    start_closure: Closure<dyn FnMut(sys::TransformStreamDefaultController) -> Promise>,
    transform_closure: Closure<dyn FnMut(JsValue, sys::TransformStreamDefaultController) -> Promise>,
    flush_closure: Closure<dyn FnMut(sys::TransformStreamDefaultController) -> Promise>,
}

impl JsTransformer {
    pub fn new(transformer: Box<dyn Transformer + 'static>) -> JsTransformer {
        let transformer = Rc::new(RefCell::new(transformer));

        let start_closure = {
            let transformer = transformer.clone();
            Closure::wrap(Box::new(move |controller: sys::TransformStreamDefaultController| {
                let transformer = transformer.clone();
                future_to_promise(async move {
                    // This mutable borrow can never panic, since the TransformStream always
                    // queues each operation on the transformer.
                    let mut transformer = transformer.borrow_mut();
                    transformer.start(&From::from(controller)).await?;
                    Ok(JsValue::undefined())
                })
            }) as Box<dyn FnMut(sys::TransformStreamDefaultController) -> Promise>)
        };
        let transform_closure = {
            let transformer = transformer.clone();
            Closure::wrap(Box::new(move |chunk: JsValue, controller: sys::TransformStreamDefaultController| {
                let transformer = transformer.clone();
                future_to_promise(async move {
                    let mut transformer = transformer.borrow_mut();
                    transformer.transform(chunk, &From::from(controller)).await?;
                    Ok(JsValue::undefined())
                })
            }) as Box<dyn FnMut(JsValue, sys::TransformStreamDefaultController) -> Promise>)
        };
        let flush_closure = {
            let transformer = transformer.clone();
            Closure::wrap(Box::new(move |controller: sys::TransformStreamDefaultController| {
                let transformer = transformer.clone();
                future_to_promise(async move {
                    let mut transformer = transformer.borrow_mut();
                    transformer.flush(&From::from(controller)).await?;
                    Ok(JsValue::undefined())
                })
            }) as Box<dyn FnMut(sys::TransformStreamDefaultController) -> Promise>)
        };

        let raw = sys::Transformer::from(JsValue::from(Object::new()));
        raw.set_start(&start_closure);
        raw.set_transform(&transform_closure);
        raw.set_flush(&flush_closure);

        JsTransformer {
            raw,
            start_closure,
            transform_closure,
            flush_closure,
        }
    }

    #[inline]
    pub fn as_raw(&self) -> &sys::Transformer {
        &self.raw
    }

    pub fn forget(self) -> sys::Transformer {
        self.start_closure.forget();
        self.transform_closure.forget();
        self.flush_closure.forget();
        self.raw
    }
}