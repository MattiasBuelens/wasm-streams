use std::pin::Pin;

use futures::future::{abortable, join, Aborted};
use futures::stream::{iter, StreamExt};
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
    let readable = ReadableStream::from_raw(new_readable_stream_from_array(
        vec![JsValue::from("Hello"), JsValue::from("world!")].into_boxed_slice(),
    ));
    assert!(!readable.is_locked());

    let mut stream = readable.into_stream().unwrap();

    assert_eq!(stream.next().await, Some(Ok(JsValue::from("Hello"))));
    assert_eq!(stream.next().await, Some(Ok(JsValue::from("world!"))));
    assert_eq!(stream.next().await, None);
}

#[wasm_bindgen_test]
fn test_readable_stream_into_stream_impl_unpin() {
    let readable = ReadableStream::from_raw(new_noop_readable_stream());
    let stream: IntoStream = readable.into_stream().unwrap();

    let _ = Pin::new(&stream); // must be Unpin for this to work
}

#[wasm_bindgen_test]
async fn test_readable_stream_reader_into_stream() {
    let mut readable = ReadableStream::from_raw(new_readable_stream_from_array(
        vec![JsValue::from("Hello"), JsValue::from("world!")].into_boxed_slice(),
    ));
    assert!(!readable.is_locked());

    {
        // Acquire a reader and wrap it in a Rust stream
        let reader = readable.get_reader().unwrap();
        let mut stream = reader.into_stream();

        assert_eq!(stream.next().await, Some(Ok(JsValue::from("Hello"))));
    }

    // Dropping the wrapped stream should release the lock
    assert!(!readable.is_locked());

    {
        // Can acquire a new reader after wrapped stream is dropped
        let mut reader = readable.get_reader().unwrap();
        assert_eq!(reader.read().await.unwrap(), Some(JsValue::from("world!")));
        assert_eq!(reader.read().await.unwrap(), None);
    }
}

#[wasm_bindgen_test]
async fn test_readable_stream_from_stream() {
    let stream = iter(vec!["Hello", "world!"]).map(|s| Ok(JsValue::from(s)));
    let mut readable = ReadableStream::from_stream(stream);

    let mut reader = readable.get_reader().unwrap();
    assert_eq!(reader.read().await.unwrap(), Some(JsValue::from("Hello")));
    assert_eq!(reader.read().await.unwrap(), Some(JsValue::from("world!")));
    assert_eq!(reader.read().await.unwrap(), None);
    reader.closed().await.unwrap();
}

#[wasm_bindgen_test]
async fn test_readable_stream_from_stream_cancel() {
    let stream = iter(vec!["Hello", "world!"]).map(|s| Ok(JsValue::from(s)));
    let mut readable = ReadableStream::from_stream(stream);

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

#[wasm_bindgen_test]
async fn test_readable_stream_from_stream_then_into_stream() {
    let stream = iter(vec!["Hello", "world!"]).map(|s| Ok(JsValue::from(s)));
    let readable = ReadableStream::from_stream(stream);

    let mut stream = readable.into_stream().unwrap();

    assert_eq!(stream.next().await, Some(Ok(JsValue::from("Hello"))));
    assert_eq!(stream.next().await, Some(Ok(JsValue::from("world!"))));
    assert_eq!(stream.next().await, None);
}

#[wasm_bindgen_test]
async fn test_readable_stream_into_stream_then_from_stream() {
    let readable = ReadableStream::from_raw(new_readable_stream_from_array(
        vec![JsValue::from("Hello"), JsValue::from("world!")].into_boxed_slice(),
    ));
    let stream = readable.into_stream().unwrap();
    let mut readable = ReadableStream::from_stream(stream);

    let mut reader = readable.get_reader().unwrap();
    assert_eq!(reader.read().await.unwrap(), Some(JsValue::from("Hello")));
    assert_eq!(reader.read().await.unwrap(), Some(JsValue::from("world!")));
    assert_eq!(reader.read().await.unwrap(), None);
    reader.closed().await.unwrap();
}
