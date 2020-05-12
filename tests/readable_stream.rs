use futures::future::{abortable, join, Aborted};
use futures::stream::{iter, Stream, StreamExt};
use pin_utils::pin_mut;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

use wasm_streams::readable::*;

#[wasm_bindgen(module = "/tests/readable_stream.js")]
extern "C" {
    fn new_noop_readable_stream() -> sys::ReadableStream;
    fn new_readable_stream_from_array(chunks: Box<[JsValue]>) -> sys::ReadableStream;
}

#[wasm_bindgen_test]
async fn test_readable_stream_new() {
    let mut readable = ReadableStream::from_raw(new_readable_stream_from_array(
        vec![JsValue::from("Hello"), JsValue::from("world!")].into_boxed_slice(),
    ));
    assert!(!readable.is_locked());

    let mut reader = readable.get_reader().unwrap();
    assert_eq!(reader.read().await.unwrap(), Some(JsValue::from("Hello")));
    assert_eq!(reader.read().await.unwrap(), Some(JsValue::from("world!")));
    assert_eq!(reader.read().await.unwrap(), None);
    reader.closed().await.unwrap();
}

#[wasm_bindgen_test]
async fn test_readable_stream_into_stream() {
    let mut readable = ReadableStream::from_raw(new_readable_stream_from_array(
        vec![JsValue::from("Hello"), JsValue::from("world!")].into_boxed_slice(),
    ));
    assert!(!readable.is_locked());

    let reader = readable.get_reader().unwrap();
    let stream = reader.into_stream();
    pin_mut!(stream);

    assert_eq!(stream.next().await, Some(Ok(JsValue::from("Hello"))));
    assert_eq!(stream.next().await, Some(Ok(JsValue::from("world!"))));
    assert_eq!(stream.next().await, None);
}

#[wasm_bindgen_test]
async fn test_readable_stream_from_stream() {
    let stream = Box::new(iter(vec!["Hello", "world!"]).map(|s| Ok(JsValue::from(s))))
        as Box<dyn Stream<Item = Result<JsValue, JsValue>>>;
    let mut readable = ReadableStream::from(stream);

    let mut reader = readable.get_reader().unwrap();
    assert_eq!(reader.read().await.unwrap(), Some(JsValue::from("Hello")));
    assert_eq!(reader.read().await.unwrap(), Some(JsValue::from("world!")));
    assert_eq!(reader.read().await.unwrap(), None);
    reader.closed().await.unwrap();
}

#[wasm_bindgen_test]
async fn test_readable_stream_from_stream_cancel() {
    let stream = Box::new(iter(vec!["Hello", "world!"]).map(|s| Ok(JsValue::from(s))))
        as Box<dyn Stream<Item = Result<JsValue, JsValue>>>;
    let mut readable = ReadableStream::from(stream);

    let mut reader = readable.get_reader().unwrap();
    assert_eq!(reader.read().await.unwrap(), Some(JsValue::from("Hello")));
    assert_eq!(reader.cancel().await, Ok(()));
    reader.closed().await.unwrap();
}

#[wasm_bindgen_test]
async fn test_readable_stream_multiple_release_lock() {
    let mut readable = ReadableStream::from_raw(new_noop_readable_stream());

    let mut reader = readable.get_reader().unwrap();
    reader.release_lock().unwrap();
    reader.release_lock().unwrap();
    reader.release_lock().unwrap();
}

#[wasm_bindgen_test]
async fn test_readable_stream_abort_read() {
    let mut readable = ReadableStream::from_raw(new_noop_readable_stream());

    let mut reader = readable.get_reader().unwrap();

    // Start reading, but abort the future immediately
    // Use `join` to poll the future at least once
    let (fut, handle) = abortable(reader.read());
    let (result, _) = join(fut, async {
        handle.abort();
    })
    .await;
    assert_eq!(result, Err(Aborted));

    // Must cancel any pending reads before releasing the reader's lock
    reader.cancel().await.unwrap();
}
