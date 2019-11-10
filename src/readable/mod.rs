use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use js_sys::{Object, Promise};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{future_to_promise, JsFuture};

use async_trait::async_trait;
pub use into_stream::IntoStream;

mod into_stream;
pub mod sys;

pub struct ReadableStream {
    raw: sys::ReadableStream,
    _source: Option<JsUnderlyingSource>,
}

impl ReadableStream {
    pub fn new(source: Box<dyn UnderlyingSource + 'static>) -> ReadableStream {
        let source = JsUnderlyingSource::new(source);
        let raw = sys::ReadableStream::new_with_source(source.as_raw());
        ReadableStream {
            raw,
            _source: Some(source),
        }
    }

    #[inline]
    pub fn as_raw(&self) -> &sys::ReadableStream {
        &self.raw
    }

    pub fn is_locked(&self) -> bool {
        self.raw.is_locked()
    }

    pub async fn cancel(&mut self) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.raw.cancel()).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub async fn cancel_with_reason(&mut self, reason: &JsValue) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.raw.cancel_with_reason(reason)).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub fn get_reader(&mut self) -> Result<ReadableStreamDefaultReader<'_>, JsValue> {
        Ok(ReadableStreamDefaultReader {
            raw: Some(self.raw.get_reader()?),
            _stream: PhantomData,
        })
    }

    pub fn forget(self) -> sys::ReadableStream {
        if let Some(source) = self._source {
            source.forget();
        }
        self.raw
    }
}

impl From<sys::ReadableStream> for ReadableStream {
    fn from(raw: sys::ReadableStream) -> ReadableStream {
        ReadableStream {
            raw,
            _source: None,
        }
    }
}

pub struct ReadableStreamDefaultController {
    raw: sys::ReadableStreamDefaultController
}

impl ReadableStreamDefaultController {
    #[inline]
    pub fn as_raw(&self) -> &sys::ReadableStreamDefaultController {
        &self.raw
    }

    pub fn desired_size(&self) -> Option<f64> {
        self.raw.desired_size()
    }

    pub fn close(&self) {
        self.raw.close()
    }

    pub fn enqueue(&self, chunk: &JsValue) {
        self.raw.enqueue(chunk)
    }

    pub fn error(&self, error: &JsValue) {
        self.raw.error(error)
    }
}

impl From<sys::ReadableStreamDefaultController> for ReadableStreamDefaultController {
    fn from(raw: sys::ReadableStreamDefaultController) -> ReadableStreamDefaultController {
        ReadableStreamDefaultController {
            raw
        }
    }
}

#[async_trait(? Send)]
pub trait UnderlyingSource {
    async fn start(&mut self, controller: &ReadableStreamDefaultController) -> Result<(), JsValue> {
        let _ = controller;
        Ok(())
    }

    async fn pull(&mut self, controller: &ReadableStreamDefaultController) -> Result<(), JsValue> {
        let _ = controller;
        Ok(())
    }

    async fn cancel(&mut self, reason: &JsValue) -> Result<(), JsValue> {
        let _ = reason;
        Ok(())
    }
}

struct JsUnderlyingSource {
    raw: sys::UnderlyingSource,
    start_closure: Closure<dyn FnMut(sys::ReadableStreamDefaultController) -> Promise>,
    pull_closure: Closure<dyn FnMut(sys::ReadableStreamDefaultController) -> Promise>,
    cancel_closure: Closure<dyn FnMut(JsValue) -> Promise>,
}

impl JsUnderlyingSource {
    pub fn new(source: Box<dyn UnderlyingSource + 'static>) -> JsUnderlyingSource {
        let source = Rc::new(RefCell::new(source));

        let start_closure = {
            let source = source.clone();
            Closure::wrap(Box::new(move |controller: sys::ReadableStreamDefaultController| {
                let source = source.clone();
                future_to_promise(async move {
                    // This mutable borrow can never panic, since the ReadableStream always
                    // queues each operation on the underlying source.
                    let mut source = source.borrow_mut();
                    source.start(&From::from(controller)).await?;
                    Ok(JsValue::undefined())
                })
            }) as Box<dyn FnMut(sys::ReadableStreamDefaultController) -> Promise>)
        };
        let pull_closure = {
            let source = source.clone();
            Closure::wrap(Box::new(move |controller: sys::ReadableStreamDefaultController| {
                let source = source.clone();
                future_to_promise(async move {
                    let mut source = source.borrow_mut();
                    source.pull(&From::from(controller)).await?;
                    Ok(JsValue::undefined())
                })
            }) as Box<dyn FnMut(sys::ReadableStreamDefaultController) -> Promise>)
        };
        let cancel_closure = {
            let source = source.clone();
            Closure::wrap(Box::new(move |reason: JsValue| {
                let source = source.clone();
                future_to_promise(async move {
                    let mut source = source.borrow_mut();
                    source.cancel(&reason).await?;
                    Ok(JsValue::undefined())
                })
            }) as Box<dyn FnMut(JsValue) -> Promise>)
        };

        let raw = sys::UnderlyingSource::from(JsValue::from(Object::new()));
        raw.set_start(&start_closure);
        raw.set_pull(&pull_closure);
        raw.set_cancel(&cancel_closure);

        JsUnderlyingSource {
            raw,
            start_closure,
            pull_closure,
            cancel_closure,
        }
    }

    #[inline]
    pub fn as_raw(&self) -> &sys::UnderlyingSource {
        &self.raw
    }

    pub fn forget(self) -> sys::UnderlyingSource {
        self.start_closure.forget();
        self.pull_closure.forget();
        self.cancel_closure.forget();
        self.raw
    }
}

pub struct ReadableStreamDefaultReader<'stream> {
    raw: Option<sys::ReadableStreamDefaultReader>,
    _stream: PhantomData<&'stream mut ReadableStream>,
}

impl<'stream> ReadableStreamDefaultReader<'stream> {
    #[inline]
    pub fn as_raw(&self) -> &sys::ReadableStreamDefaultReader {
        self.raw.as_ref().unwrap()
    }

    pub async fn closed(&self) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.as_raw().closed()).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub async fn cancel(&mut self) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.as_raw().cancel()).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub async fn cancel_with_reason(&mut self, reason: &JsValue) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.as_raw().cancel_with_reason(reason)).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub async fn read(&mut self) -> Result<Option<JsValue>, JsValue> {
        let js_value = JsFuture::from(self.as_raw().read()).await?;
        let result = sys::ReadableStreamReadResult::from(js_value);
        if result.is_done() {
            Ok(None)
        } else {
            Ok(Some(result.value()))
        }
    }

    pub fn release_lock(&mut self) -> Result<(), JsValue> {
        if let Some(raw) = self.raw.as_ref() {
            raw.release_lock()?;
            self.raw.take();
        }
        Ok(())
    }

    pub fn into_stream(self) -> IntoStream<'stream> {
        IntoStream::new(self)
    }
}

impl Drop for ReadableStreamDefaultReader<'_> {
    fn drop(&mut self) {
        // TODO Error handling?
        self.release_lock().unwrap();
    }
}
