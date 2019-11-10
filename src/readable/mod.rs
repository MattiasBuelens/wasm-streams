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
    inner: sys::ReadableStream,
    _source: Option<JsUnderlyingSource>,
}

impl ReadableStream {
    pub fn new(source: Box<dyn UnderlyingSource + 'static>) -> ReadableStream {
        let source = JsUnderlyingSource::new(source);
        let inner = sys::ReadableStream::new_with_source(source.as_raw());
        ReadableStream {
            inner,
            _source: Some(source),
        }
    }

    #[inline]
    pub fn as_raw(&self) -> &sys::ReadableStream {
        &self.inner
    }

    pub fn is_locked(&self) -> bool {
        self.inner.is_locked()
    }

    pub async fn cancel(&mut self) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.inner.cancel()).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub async fn cancel_with_reason(&mut self, reason: &JsValue) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.inner.cancel_with_reason(reason)).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub fn get_reader(&mut self) -> Result<ReadableStreamDefaultReader<'_>, JsValue> {
        Ok(ReadableStreamDefaultReader {
            inner: Some(self.inner.get_reader()?),
            _stream: PhantomData,
        })
    }

    pub fn forget(self) -> sys::ReadableStream {
        if let Some(source) = self._source {
            source.forget();
        }
        self.inner
    }
}

impl From<sys::ReadableStream> for ReadableStream {
    fn from(raw: sys::ReadableStream) -> ReadableStream {
        ReadableStream {
            inner: raw,
            _source: None,
        }
    }
}

pub struct ReadableStreamDefaultController {
    inner: sys::ReadableStreamDefaultController
}

impl ReadableStreamDefaultController {
    #[inline]
    pub fn as_raw(&self) -> &sys::ReadableStreamDefaultController {
        &self.inner
    }

    pub fn desired_size(&self) -> Option<f64> {
        self.inner.desired_size()
    }

    pub fn close(&self) {
        self.inner.close()
    }

    pub fn enqueue(&self, chunk: &JsValue) {
        self.inner.enqueue(chunk)
    }

    pub fn error(&self, error: &JsValue) {
        self.inner.error(error)
    }
}

impl From<sys::ReadableStreamDefaultController> for ReadableStreamDefaultController {
    fn from(raw: sys::ReadableStreamDefaultController) -> ReadableStreamDefaultController {
        ReadableStreamDefaultController {
            inner: raw
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
    inner: sys::UnderlyingSource,
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

        let inner = sys::UnderlyingSource::from(JsValue::from(Object::new()));
        inner.set_start(&start_closure);
        inner.set_pull(&pull_closure);
        inner.set_cancel(&cancel_closure);

        JsUnderlyingSource {
            inner,
            start_closure,
            pull_closure,
            cancel_closure,
        }
    }

    #[inline]
    pub fn as_raw(&self) -> &sys::UnderlyingSource {
        &self.inner
    }

    pub fn forget(self) -> sys::UnderlyingSource {
        self.start_closure.forget();
        self.pull_closure.forget();
        self.cancel_closure.forget();
        self.inner
    }
}

pub struct ReadableStreamDefaultReader<'stream> {
    inner: Option<sys::ReadableStreamDefaultReader>,
    _stream: PhantomData<&'stream mut ReadableStream>,
}

impl<'stream> ReadableStreamDefaultReader<'stream> {
    #[inline]
    pub fn as_raw(&self) -> &sys::ReadableStreamDefaultReader {
        self.inner.as_ref().unwrap()
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
        if let Some(inner) = self.inner.as_ref() {
            inner.release_lock()?;
            self.inner.take();
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
