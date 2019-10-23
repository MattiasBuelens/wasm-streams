use futures::{Stream, TryFutureExt, TryStreamExt};
use futures::stream::unfold;
use js_sys::Object;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

use crate::sys::{
    ReadableStream as RawReadableStream,
    ReadableStreamDefaultController,
    ReadableStreamDefaultReader as RawReadableStreamDefaultReader,
    ReadableStreamReadResult,
    UnderlyingSource as RawUnderlyingSource,
};

pub struct ReadableStream {
    inner: RawReadableStream
}

impl ReadableStream {
    pub fn new(source: UnderlyingSource) -> ReadableStream {
        let inner = RawReadableStream::new_with_source(source.as_raw());
        ReadableStream {
            inner
        }
    }

    pub fn as_raw(&self) -> &RawReadableStream {
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

    pub fn get_reader(&mut self) -> Result<ReadableStreamDefaultReader, JsValue> {
        Ok(ReadableStreamDefaultReader {
            inner: self.inner.get_reader()?
        })
    }
}

pub struct UnderlyingSource {
    inner: RawUnderlyingSource,
    start_closure: Option<Closure<dyn FnMut(&ReadableStreamDefaultController)>>,
    pull_closure: Option<Closure<dyn FnMut(&ReadableStreamDefaultController)>>,
    cancel_closure: Option<Closure<dyn FnMut(&JsValue)>>,
}

impl UnderlyingSource {
    pub fn new(
        start_cb: Option<Box<dyn FnMut(&ReadableStreamDefaultController)>>,
        pull_cb: Option<Box<dyn FnMut(&ReadableStreamDefaultController)>>,
        cancel_cb: Option<Box<dyn FnMut(&JsValue)>>,
    ) -> UnderlyingSource {
        let inner = RawUnderlyingSource::from(JsValue::from(Object::new()));

        let start_closure = start_cb.map(|cb| {
            let closure = Closure::wrap(cb);
            inner.set_start(&closure);
            closure
        });
        let pull_closure = pull_cb.map(|cb| {
            let closure = Closure::wrap(cb);
            inner.set_pull(&closure);
            closure
        });
        let cancel_closure = cancel_cb.map(|cb| {
            let closure = Closure::wrap(cb);
            inner.set_cancel(&closure);
            closure
        });

        UnderlyingSource {
            inner,
            start_closure,
            pull_closure,
            cancel_closure,
        }
    }

    pub fn as_raw(&self) -> &RawUnderlyingSource {
        &self.inner
    }

    pub fn forget(mut self) {
        self.start_closure.take().map(|closure| closure.forget());
        self.pull_closure.take().map(|closure| closure.forget());
        self.cancel_closure.take().map(|closure| closure.forget());
    }
}

pub struct ReadableStreamDefaultReader {
    inner: RawReadableStreamDefaultReader
}

impl ReadableStreamDefaultReader {
    pub async fn closed(&self) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.inner.closed()).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
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

    pub async fn read(&mut self) -> Result<Option<JsValue>, JsValue> {
        let js_value = JsFuture::from(self.inner.read()).await?;
        let result = ReadableStreamReadResult::from(js_value);
        if result.is_done() {
            Ok(None)
        } else {
            Ok(Some(result.value()))
        }
    }

    pub fn release_lock(&mut self) -> Result<(), JsValue> {
        self.inner.release_lock()?;
        Ok(())
    }
}

impl ReadableStream {
    pub fn into_stream(self) -> impl Stream<Item=Result<JsValue, JsValue>> {
        self.into_stream_fut().try_flatten_stream().into_stream()
    }

    async fn into_stream_fut(mut self) -> Result<impl Stream<Item=Result<JsValue, JsValue>>, JsValue> {
        let reader = self.get_reader()?;
        let stream = unfold(reader, |mut reader| async move {
            match reader.read().await {
                Ok(Some(value)) => Some((Ok(value), reader)),
                Ok(None) => None,
                Err(error) => Some((Err(error), reader))
            }
        });
        Ok(stream)
    }
}
