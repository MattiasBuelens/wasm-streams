use std::marker::PhantomData;

use futures::stream::Stream;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

pub use into_stream::IntoStream;
use into_underlying_source::IntoUnderlyingSource;

mod into_stream;
mod into_underlying_source;
pub mod sys;

pub struct ReadableStream {
    raw: sys::ReadableStream,
}

impl ReadableStream {
    #[inline]
    pub fn as_raw(&self) -> &sys::ReadableStream {
        &self.raw
    }

    #[inline]
    pub fn into_raw(self) -> sys::ReadableStream {
        self.raw
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
}

impl From<sys::ReadableStream> for ReadableStream {
    fn from(raw: sys::ReadableStream) -> ReadableStream {
        ReadableStream { raw }
    }
}

impl From<Box<dyn Stream<Item = Result<JsValue, JsValue>>>> for ReadableStream {
    fn from(stream: Box<dyn Stream<Item = Result<JsValue, JsValue>>>) -> Self {
        ReadableStream::from(sys::ReadableStream::new_with_source(
            IntoUnderlyingSource::new(stream),
        ))
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
