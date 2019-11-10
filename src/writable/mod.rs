use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use js_sys::{Object, Promise};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{future_to_promise, JsFuture};

use async_trait::async_trait;
pub use into_sink::IntoSink;

mod into_sink;
pub mod sys;

pub struct WritableStream {
    raw: sys::WritableStream,
    _sink: Option<JsUnderlyingSink>,
}

impl WritableStream {
    pub fn new(sink: Box<dyn UnderlyingSink + 'static>) -> WritableStream {
        let sink = JsUnderlyingSink::new(sink);
        let raw = sys::WritableStream::new_with_sink(sink.as_raw());
        WritableStream {
            raw,
            _sink: Some(sink),
        }
    }

    #[inline]
    pub fn as_raw(&self) -> &sys::WritableStream {
        &self.raw
    }

    pub fn is_locked(&self) -> bool {
        self.raw.is_locked()
    }

    pub async fn abort(&mut self) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.raw.abort()).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub async fn abort_with_reason(&mut self, reason: &JsValue) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.raw.abort_with_reason(reason)).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub fn get_writer(&mut self) -> Result<WritableStreamDefaultWriter<'_>, JsValue> {
        Ok(WritableStreamDefaultWriter {
            raw: Some(self.raw.get_writer()?),
            _stream: PhantomData,
        })
    }

    pub fn forget(self) -> sys::WritableStream {
        if let Some(sink) = self._sink {
            sink.forget();
        }
        self.raw
    }
}

impl From<sys::WritableStream> for WritableStream {
    fn from(raw: sys::WritableStream) -> WritableStream {
        WritableStream {
            raw,
            _sink: None,
        }
    }
}

pub struct WritableStreamDefaultController {
    raw: sys::WritableStreamDefaultController
}

impl WritableStreamDefaultController {
    #[inline]
    pub fn as_raw(&self) -> &sys::WritableStreamDefaultController {
        &self.raw
    }

    pub fn error(&self, error: &JsValue) {
        self.raw.error(error)
    }
}

impl From<sys::WritableStreamDefaultController> for WritableStreamDefaultController {
    fn from(raw: sys::WritableStreamDefaultController) -> WritableStreamDefaultController {
        WritableStreamDefaultController {
            raw
        }
    }
}

#[async_trait(? Send)]
pub trait UnderlyingSink {
    async fn start(&mut self, controller: &WritableStreamDefaultController) -> Result<(), JsValue> {
        let _ = controller;
        Ok(())
    }

    async fn write(&mut self, chunk: JsValue, controller: &WritableStreamDefaultController) -> Result<(), JsValue> {
        let _ = (chunk, controller);
        Ok(())
    }

    async fn close(&mut self) -> Result<(), JsValue> {
        Ok(())
    }

    async fn abort(&mut self, reason: &JsValue) -> Result<(), JsValue> {
        let _ = reason;
        Ok(())
    }
}

struct JsUnderlyingSink {
    raw: sys::UnderlyingSink,
    start_closure: Closure<dyn FnMut(sys::WritableStreamDefaultController) -> Promise>,
    write_closure: Closure<dyn FnMut(JsValue, sys::WritableStreamDefaultController) -> Promise>,
    close_closure: Closure<dyn FnMut() -> Promise>,
    abort_closure: Closure<dyn FnMut(JsValue) -> Promise>,
}

impl JsUnderlyingSink {
    pub fn new(sink: Box<dyn UnderlyingSink + 'static>) -> JsUnderlyingSink {
        let sink = Rc::new(RefCell::new(sink));

        let start_closure = {
            let sink = sink.clone();
            Closure::wrap(Box::new(move |controller: sys::WritableStreamDefaultController| {
                let sink = sink.clone();
                future_to_promise(async move {
                    // This mutable borrow can never panic, since the WritableStream always
                    // queues each operation on the underlying sink.
                    let mut sink = sink.borrow_mut();
                    sink.start(&From::from(controller)).await?;
                    Ok(JsValue::undefined())
                })
            }) as Box<dyn FnMut(sys::WritableStreamDefaultController) -> Promise>)
        };
        let write_closure = {
            let sink = sink.clone();
            Closure::wrap(Box::new(move |chunk: JsValue, controller: sys::WritableStreamDefaultController| {
                let sink = sink.clone();
                future_to_promise(async move {
                    let mut sink = sink.borrow_mut();
                    sink.write(chunk, &From::from(controller)).await?;
                    Ok(JsValue::undefined())
                })
            }) as Box<dyn FnMut(JsValue, sys::WritableStreamDefaultController) -> Promise>)
        };
        let close_closure = {
            let sink = sink.clone();
            Closure::wrap(Box::new(move || {
                let sink = sink.clone();
                future_to_promise(async move {
                    let mut sink = sink.borrow_mut();
                    sink.close().await?;
                    Ok(JsValue::undefined())
                })
            }) as Box<dyn FnMut() -> Promise>)
        };
        let abort_closure = {
            let sink = sink.clone();
            Closure::wrap(Box::new(move |reason: JsValue| {
                let sink = sink.clone();
                future_to_promise(async move {
                    let mut sink = sink.borrow_mut();
                    sink.abort(&reason).await?;
                    Ok(JsValue::undefined())
                })
            }) as Box<dyn FnMut(JsValue) -> Promise>)
        };

        let raw = sys::UnderlyingSink::from(JsValue::from(Object::new()));
        raw.set_start(&start_closure);
        raw.set_write(&write_closure);
        raw.set_close(&close_closure);
        raw.set_abort(&abort_closure);

        JsUnderlyingSink {
            raw,
            start_closure,
            write_closure,
            close_closure,
            abort_closure,
        }
    }

    #[inline]
    pub fn as_raw(&self) -> &sys::UnderlyingSink {
        &self.raw
    }

    pub fn forget(self) -> sys::UnderlyingSink {
        self.start_closure.forget();
        self.write_closure.forget();
        self.close_closure.forget();
        self.abort_closure.forget();
        self.raw
    }
}

pub struct WritableStreamDefaultWriter<'stream> {
    raw: Option<sys::WritableStreamDefaultWriter>,
    _stream: PhantomData<&'stream mut WritableStream>,
}

impl<'stream> WritableStreamDefaultWriter<'stream> {
    #[inline]
    pub fn as_raw(&self) -> &sys::WritableStreamDefaultWriter {
        self.raw.as_ref().unwrap()
    }

    pub async fn closed(&self) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.as_raw().closed()).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub fn desired_size(&self) -> Option<f64> {
        self.as_raw().desired_size()
    }

    pub async fn ready(&self) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.as_raw().ready()).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub async fn abort(&mut self) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.as_raw().abort()).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub async fn abort_with_reason(&mut self, reason: &JsValue) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.as_raw().abort_with_reason(reason)).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub async fn write(&mut self, chunk: JsValue) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.as_raw().write(chunk)).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub async fn close(&mut self) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.as_raw().close()).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub fn release_lock(&mut self) -> Result<(), JsValue> {
        if let Some(raw) = self.raw.as_ref() {
            raw.release_lock()?;
            self.raw.take();
        }
        Ok(())
    }

    pub fn into_sink(self) -> IntoSink<'stream> {
        IntoSink::new(self)
    }
}

impl Drop for WritableStreamDefaultWriter<'_> {
    fn drop(&mut self) {
        // TODO Error handling?
        self.release_lock().unwrap();
    }
}
