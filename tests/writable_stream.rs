extern crate wasm_bindgen_test;

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

use async_trait::async_trait;
use wasm_streams::writable_stream::*;

struct NoopSink;

#[async_trait(? Send)]
impl UnderlyingSink for NoopSink {}

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
