use futures::future::join;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

use async_trait::async_trait;
use wasm_streams::transform::*;

struct NoopTransformer;

struct UppercaseTransformer;

#[async_trait(? Send)]
impl Transformer for NoopTransformer {}

#[async_trait(? Send)]
impl Transformer for UppercaseTransformer {
    async fn transform(&mut self, chunk: JsValue, controller: &TransformStreamDefaultController) -> Result<(), JsValue> {
        let mut value = chunk.as_string().unwrap();
        value.make_ascii_uppercase();
        controller.enqueue(&JsValue::from(value));
        Ok(())
    }
}

#[wasm_bindgen_test]
async fn test_transform_stream_new() {
    let transform = TransformStream::new(Box::new(NoopTransformer));
    join(async {
        let mut writable = transform.writable();
        let mut writer = writable.get_writer().unwrap();
        writer.write(JsValue::from("Hello")).await.unwrap();
        writer.write(JsValue::from("world!")).await.unwrap();
        writer.close().await.unwrap();
    }, async {
        let mut readable = transform.readable();
        let mut reader = readable.get_reader().unwrap();
        assert_eq!(reader.read().await.unwrap(), Some(JsValue::from("Hello")));
        assert_eq!(reader.read().await.unwrap(), Some(JsValue::from("world!")));
        assert_eq!(reader.read().await.unwrap(), None);
    }).await;
}

#[wasm_bindgen_test]
async fn test_transform_stream_new_uppercase() {
    let transform = TransformStream::new(Box::new(UppercaseTransformer));
    join(async {
        let mut writable = transform.writable();
        let mut writer = writable.get_writer().unwrap();
        writer.write(JsValue::from("Hello")).await.unwrap();
        writer.write(JsValue::from("world!")).await.unwrap();
        writer.close().await.unwrap();
    }, async {
        let mut readable = transform.readable();
        let mut reader = readable.get_reader().unwrap();
        assert_eq!(reader.read().await.unwrap(), Some(JsValue::from("HELLO")));
        assert_eq!(reader.read().await.unwrap(), Some(JsValue::from("WORLD!")));
        assert_eq!(reader.read().await.unwrap(), None);
    }).await;
}
