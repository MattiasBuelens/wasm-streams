#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

use wasm_streams::bindings::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_readable_stream_new() {
    let start_cb = Closure::once(move |controller: &ReadableStreamDefaultController| {
        controller.enqueue(&JsValue::from("Hello"));
        controller.enqueue(&JsValue::from("world!"));
        controller.close();
    });
    let source = UnderlyingSource::new();
    source.set_start(&start_cb);

    let readable = ReadableStream::new_with_source(&source);

    let reader = readable.get_reader().unwrap();
    assert_eq!(reader.read().await.unwrap(), Some(JsValue::from("Hello")));
    assert_eq!(reader.read().await.unwrap(), Some(JsValue::from("world!")));
    assert_eq!(reader.read().await.unwrap(), None);
}
