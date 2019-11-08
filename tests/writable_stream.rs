extern crate wasm_bindgen_test;

use futures::sink::SinkExt;
use pin_utils::pin_mut;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

use async_trait::async_trait;
use wasm_streams::writable_stream::*;

struct NoopSink;

struct ConsoleSink;

#[async_trait(? Send)]
impl UnderlyingSink for NoopSink {}

#[async_trait(? Send)]
impl UnderlyingSink for ConsoleSink {
    async fn write(&mut self, chunk: JsValue, _: &WritableStreamDefaultController) -> Result<(), JsValue> {
        console_log!("wrote chunk: {}", chunk.as_string().unwrap());
        Ok(())
    }
    async fn close(&mut self) -> Result<(), JsValue> {
        console_log!("close");
        Ok(())
    }
}

#[wasm_bindgen_test]
async fn test_writable_stream_new() {
    let mut writable = WritableStream::new(Box::new(NoopSink));
    assert!(!writable.is_locked());

    let mut writer = writable.get_writer().unwrap();
    assert_eq!(writer.write(JsValue::from("Hello")).await.unwrap(), ());
    assert_eq!(writer.write(JsValue::from("world!")).await.unwrap(), ());
    assert_eq!(writer.close().await.unwrap(), ());
    writer.closed().await.unwrap();
}

#[wasm_bindgen_test]
async fn test_writable_stream_into_sink() {
    let mut writable = WritableStream::new(Box::new(ConsoleSink));
    assert!(!writable.is_locked());

    let writer = writable.get_writer().unwrap();
    let sink = writer.into_sink();
    pin_mut!(sink);
    assert_eq!(sink.send(JsValue::from("Hello")).await, Ok(()));
    assert_eq!(sink.send(JsValue::from("world!")).await, Ok(()));
    assert_eq!(sink.close().await, Ok(()));
}
