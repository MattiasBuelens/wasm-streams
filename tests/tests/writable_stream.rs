use std::pin::Pin;

use futures::channel::*;
use futures::stream::iter;
use futures::{AsyncWriteExt, SinkExt, StreamExt};
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

use wasm_streams::writable::*;

use crate::js::*;

#[wasm_bindgen_test]
async fn test_writable_stream_new() {
    let mut writable = WritableStream::from_raw(new_noop_writable_stream());
    assert!(!writable.is_locked());

    let mut writer = writable.get_writer();
    assert_eq!(writer.write(JsValue::from("Hello")).await, Ok(()));
    assert_eq!(writer.write(JsValue::from("world!")).await, Ok(()));
    assert_eq!(writer.close().await, Ok(()));
    writer.closed().await.unwrap();
}

#[wasm_bindgen_test]
async fn test_writable_stream_into_sink() {
    let recording_stream = RecordingWritableStream::new();
    let writable = WritableStream::from_raw(recording_stream.stream());
    assert!(!writable.is_locked());

    let mut sink = writable.into_sink();

    assert_eq!(sink.send(JsValue::from("Hello")).await, Ok(()));
    assert_eq!(sink.send(JsValue::from("world!")).await, Ok(()));
    assert_eq!(sink.close().await, Ok(()));

    assert_eq!(
        recording_stream.events(),
        [
            RecordedEvent::Write(JsValue::from("Hello")),
            RecordedEvent::Write(JsValue::from("world!")),
            RecordedEvent::Close
        ]
    );
}

#[wasm_bindgen_test]
fn test_writable_stream_into_sink_impl_unpin() {
    let writable = WritableStream::from_raw(new_noop_writable_stream());
    let sink: IntoSink = writable.into_sink();

    let _ = Pin::new(&sink); // must be Unpin for this to work
}

#[wasm_bindgen_test]
async fn test_writable_stream_writer_into_sink() {
    let recording_stream = RecordingWritableStream::new();
    let mut writable = WritableStream::from_raw(recording_stream.stream());
    assert!(!writable.is_locked());

    {
        // Acquire a writer and wrap it in a Rust sink
        let writer = writable.get_writer();
        let mut sink = writer.into_sink();

        assert_eq!(sink.send(JsValue::from("Hello")).await, Ok(()));
    }

    assert_eq!(
        recording_stream.events(),
        [RecordedEvent::Write(JsValue::from("Hello")),]
    );

    // Dropping the wrapped sink should release the lock
    assert!(!writable.is_locked());

    {
        // Can acquire a new writer after wrapped sink is dropped
        let mut writer = writable.get_writer();
        assert_eq!(writer.write(JsValue::from("world!")).await, Ok(()));
        assert_eq!(writer.close().await, Ok(()));
    }

    assert_eq!(
        recording_stream.events(),
        [
            RecordedEvent::Write(JsValue::from("Hello")),
            RecordedEvent::Write(JsValue::from("world!")),
            RecordedEvent::Close
        ]
    );
}

#[wasm_bindgen_test]
async fn test_writable_stream_from_sink() {
    let (sink, stream) = mpsc::unbounded::<JsValue>();
    let sink = sink.sink_map_err(|_| JsValue::from_str("cannot happen"));
    let mut writable = WritableStream::from_sink(sink);

    let mut writer = writable.get_writer();
    assert_eq!(writer.write(JsValue::from("Hello")).await, Ok(()));
    assert_eq!(writer.write(JsValue::from("world!")).await, Ok(()));
    assert_eq!(writer.close().await, Ok(()));
    writer.closed().await.unwrap();

    let output = stream.collect::<Vec<_>>().await;
    assert_eq!(
        output,
        vec![JsValue::from("Hello"), JsValue::from("world!")]
    );
}

#[wasm_bindgen_test]
async fn test_writable_stream_from_sink_then_into_sink() {
    let (sink, stream) = mpsc::unbounded::<JsValue>();
    let sink = sink.sink_map_err(|_| JsValue::from_str("cannot happen"));
    let writable = WritableStream::from_sink(sink);
    let mut sink = writable.into_sink();

    let chunks = vec![JsValue::from("Hello"), JsValue::from("world!")];
    let mut input = iter(chunks.clone()).map(Ok);
    sink.send_all(&mut input).await.unwrap();
    sink.close().await.unwrap();

    let output = stream.collect::<Vec<_>>().await;
    assert_eq!(output, chunks);
}

#[wasm_bindgen_test]
async fn test_writable_stream_multiple_writers() {
    let recording_stream = RecordingWritableStream::new();
    let mut writable = WritableStream::from_raw(recording_stream.stream());

    let mut writer = writable.get_writer();
    writer.write(JsValue::from_str("Hello")).await.unwrap();
    drop(writer);

    let mut writer = writable.get_writer();
    writer.write(JsValue::from_str("world!")).await.unwrap();
    writer.close().await.unwrap();
    drop(writer);

    assert_eq!(
        recording_stream.events(),
        [
            RecordedEvent::Write(JsValue::from("Hello")),
            RecordedEvent::Write(JsValue::from("world!")),
            RecordedEvent::Close
        ]
    );
}

#[wasm_bindgen_test]
async fn test_writable_stream_into_async_write() {
    let recording_stream = RecordingWritableStream::new();
    let writable = WritableStream::from_raw(recording_stream.stream());
    assert!(!writable.is_locked());

    let mut async_write = writable.into_async_write();

    let mut buf = [1, 2, 3];
    assert_eq!(async_write.write(&buf).await.unwrap(), 3);
    buf = [4, 5, 6];
    assert_eq!(async_write.write(&buf).await.unwrap(), 3);
    buf = [7, 8, 9];
    assert_eq!(async_write.write(&buf[0..2]).await.unwrap(), 2);
    async_write.close().await.unwrap();

    assert_eq!(
        recording_stream.events(),
        [
            RecordedEvent::Write(Uint8Array::from(&[1, 2, 3][..]).into()),
            RecordedEvent::Write(Uint8Array::from(&[4, 5, 6][..]).into()),
            RecordedEvent::Write(Uint8Array::from(&[7, 8][..]).into()),
            RecordedEvent::Close
        ]
    );
}
