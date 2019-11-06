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
    let mut readable = WritableStream::new(Box::new(NoopSink));
    assert!(!readable.is_locked());

    let mut reader = readable.get_writer().unwrap();
    assert_eq!(reader.write(JsValue::from("Hello")).await.unwrap(), ());
    assert_eq!(reader.write(JsValue::from("world!")).await.unwrap(), ());
    assert_eq!(reader.close().await.unwrap(), ());
    reader.closed().await.unwrap();
}
