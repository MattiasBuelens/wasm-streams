use futures::future::{abortable, Aborted, join};
use futures::stream::StreamExt;
use pin_utils::pin_mut;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

use async_trait::async_trait;
use wasm_streams::readable_stream::*;

struct NoopSource;

struct HelloWorldSource;

#[async_trait(? Send)]
impl UnderlyingSource for NoopSource {}

#[async_trait(? Send)]
impl UnderlyingSource for HelloWorldSource {
    async fn start(&mut self, controller: &ReadableStreamDefaultController) -> Result<(), JsValue> {
        controller.enqueue(&JsValue::from("Hello"));
        controller.enqueue(&JsValue::from("world!"));
        controller.close();
        Ok(())
    }
}

#[wasm_bindgen_test]
async fn test_readable_stream_new() {
    let mut readable = ReadableStream::new(Box::new(HelloWorldSource));
    assert!(!readable.is_locked());

    let mut reader = readable.get_reader().unwrap();
    assert_eq!(reader.read().await.unwrap(), Some(JsValue::from("Hello")));
    assert_eq!(reader.read().await.unwrap(), Some(JsValue::from("world!")));
    assert_eq!(reader.read().await.unwrap(), None);
    reader.closed().await.unwrap();
}

#[wasm_bindgen_test]
async fn test_readable_stream_into_stream() {
    let mut readable = ReadableStream::new(Box::new(HelloWorldSource));
    assert!(!readable.is_locked());

    let reader = readable.get_reader().unwrap();
    let stream = reader.into_stream();
    pin_mut!(stream);

    assert_eq!(stream.next().await, Some(Ok(JsValue::from("Hello"))));
    assert_eq!(stream.next().await, Some(Ok(JsValue::from("world!"))));
    assert_eq!(stream.next().await, None);
}

#[wasm_bindgen_test]
async fn test_readable_stream_multiple_release_lock() {
    let mut readable = ReadableStream::new(Box::new(NoopSource));

    let mut reader = readable.get_reader().unwrap();
    reader.release_lock().unwrap();
    reader.release_lock().unwrap();
    reader.release_lock().unwrap();
}

#[wasm_bindgen_test]
async fn test_readable_stream_abort_read() {
    let mut readable = ReadableStream::new(Box::new(NoopSource));

    let mut reader = readable.get_reader().unwrap();

    // Start reading, but abort the future immediately
    // Use `join` to poll the future at least once
    let (fut, handle) = abortable(reader.read());
    let (result, _) = join(fut, async {
        handle.abort();
    }).await;
    assert_eq!(result, Err(Aborted));

    // Must cancel any pending reads before releasing the reader's lock
    reader.cancel().await.unwrap();
}