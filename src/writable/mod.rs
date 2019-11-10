use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use js_sys::{Object, Promise};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::{future_to_promise, JsFuture};

use async_trait::async_trait;
pub use into_sink::IntoSink;
pub use sys::WritableStreamDefaultController;

mod into_sink;
pub mod sys;

pub struct WritableStream {
    inner: sys::WritableStream,
    _sink: Option<JsUnderlyingSink>,
}

impl WritableStream {
    pub fn new(sink: Box<dyn UnderlyingSink + 'static>) -> WritableStream {
        let sink = JsUnderlyingSink::new(sink);
        let inner = sys::WritableStream::new_with_sink(sink.as_raw());
        WritableStream {
            inner,
            _sink: Some(sink),
        }
    }

    #[inline]
    pub fn as_raw(&self) -> &sys::WritableStream {
        &self.inner
    }

    pub fn is_locked(&self) -> bool {
        self.inner.is_locked()
    }

    pub async fn abort(&mut self) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.inner.abort()).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub async fn abort_with_reason(&mut self, reason: &JsValue) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.inner.abort_with_reason(reason)).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub fn get_writer(&mut self) -> Result<WritableStreamDefaultWriter<'_>, JsValue> {
        Ok(WritableStreamDefaultWriter {
            inner: Some(self.inner.get_writer()?),
            _stream: PhantomData,
        })
    }

    pub fn forget(self) -> sys::WritableStream {
        if let Some(sink) = self._sink {
            sink.forget();
        }
        self.inner
    }
}

impl From<sys::WritableStream> for WritableStream {
    fn from(raw: sys::WritableStream) -> WritableStream {
        WritableStream {
            inner: raw,
            _sink: None,
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
    inner: sys::UnderlyingSink,
    start_closure: Closure<dyn FnMut(WritableStreamDefaultController) -> Promise>,
    write_closure: Closure<dyn FnMut(JsValue, WritableStreamDefaultController) -> Promise>,
    close_closure: Closure<dyn FnMut() -> Promise>,
    abort_closure: Closure<dyn FnMut(JsValue) -> Promise>,
}

impl JsUnderlyingSink {
    pub fn new(sink: Box<dyn UnderlyingSink + 'static>) -> JsUnderlyingSink {
        let sink = Rc::new(RefCell::new(sink));

        let start_closure = {
            let sink = sink.clone();
            Closure::wrap(Box::new(move |controller: WritableStreamDefaultController| {
                let sink = sink.clone();
                future_to_promise(async move {
                    // This mutable borrow can never panic, since the WritableStream always
                    // queues each operation on the underlying sink.
                    let mut sink = sink.borrow_mut();
                    sink.start(&controller).await?;
                    Ok(JsValue::undefined())
                })
            }) as Box<dyn FnMut(WritableStreamDefaultController) -> Promise>)
        };
        let write_closure = {
            let sink = sink.clone();
            Closure::wrap(Box::new(move |chunk: JsValue, controller: WritableStreamDefaultController| {
                let sink = sink.clone();
                future_to_promise(async move {
                    let mut sink = sink.borrow_mut();
                    sink.write(chunk, &controller).await?;
                    Ok(JsValue::undefined())
                })
            }) as Box<dyn FnMut(JsValue, WritableStreamDefaultController) -> Promise>)
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

        let inner = sys::UnderlyingSink::from(JsValue::from(Object::new()));
        inner.set_start(&start_closure);
        inner.set_write(&write_closure);
        inner.set_close(&close_closure);
        inner.set_abort(&abort_closure);

        JsUnderlyingSink {
            inner,
            start_closure,
            write_closure,
            close_closure,
            abort_closure,
        }
    }

    #[inline]
    pub fn as_raw(&self) -> &sys::UnderlyingSink {
        &self.inner
    }

    pub fn forget(self) -> sys::UnderlyingSink {
        self.start_closure.forget();
        self.write_closure.forget();
        self.close_closure.forget();
        self.abort_closure.forget();
        self.inner
    }
}

pub struct WritableStreamDefaultWriter<'stream> {
    inner: Option<sys::WritableStreamDefaultWriter>,
    _stream: PhantomData<&'stream mut WritableStream>,
}

impl<'stream> WritableStreamDefaultWriter<'stream> {
    #[inline]
    pub fn as_raw(&self) -> &sys::WritableStreamDefaultWriter {
        self.inner.as_ref().unwrap()
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
        if let Some(inner) = self.inner.as_ref() {
            inner.release_lock()?;
            self.inner.take();
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
