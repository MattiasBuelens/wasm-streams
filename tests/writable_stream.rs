use std::pin::Pin;

use futures::channel::*;
use futures::stream::iter;
use futures::{SinkExt, StreamExt};
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

use wasm_streams::writable::*;

#[wasm_bindgen(module = "/tests/writable_stream.js")]
extern "C" {
    fn new_noop_writable_stream() -> sys::WritableStream;
    fn new_logging_writable_stream() -> sys::WritableStream;
}

#[wasm_bindgen_test]
async fn test_writable_stream_new() {
    let mut writable = WritableStream::from_raw(new_noop_writable_stream());
    assert!(!writable.is_locked());

    let mut writer = writable.get_writer().unwrap();
    assert_eq!(writer.write(JsValue::from("Hello")).await.unwrap(), ());
    assert_eq!(writer.write(JsValue::from("world!")).await.unwrap(), ());
    assert_eq!(writer.close().await.unwrap(), ());
    writer.closed().await.unwrap();
}

#[wasm_bindgen_test]
async fn test_writable_stream_into_sink() {
    let writable = WritableStream::from_raw(new_logging_writable_stream());
    assert!(!writable.is_locked());

    let mut sink = writable.into_sink();

    assert_eq!(sink.send(JsValue::from("Hello")).await, Ok(()));
    assert_eq!(sink.send(JsValue::from("world!")).await, Ok(()));
    assert_eq!(sink.close().await, Ok(()));
}

#[wasm_bindgen_test]
fn test_writable_stream_into_sink_impl_unpin() {
    let writable = WritableStream::from_raw(new_noop_writable_stream());
    let sink: IntoSink = writable.into_sink();

    let _ = Pin::new(&sink); // must be Unpin for this to work
}

#[wasm_bindgen_test]
async fn test_writable_stream_writer_into_sink() {
    let mut writable = WritableStream::from_raw(new_logging_writable_stream());
    assert!(!writable.is_locked());

    {
        // Acquire a writer and wrap it in a Rust sink
        let writer = writable.get_writer().unwrap();
        let mut sink = writer.into_sink();

        assert_eq!(sink.send(JsValue::from("Hello")).await, Ok(()));
    }

    // Dropping the wrapped sink should release the lock
    assert!(!writable.is_locked());

    {
        // Can acquire a new writer after wrapped sink is dropped
        let mut writer = writable.get_writer().unwrap();
        assert_eq!(writer.write(JsValue::from("world!")).await.unwrap(), ());
        assert_eq!(writer.close().await.unwrap(), ());
    }
}

#[wasm_bindgen_test]
async fn test_writable_stream_from_sink() {
    let (sink, stream) = mpsc::unbounded::<JsValue>();
    let sink = sink.sink_map_err(|_| JsValue::from_str("cannot happen"));
    let mut writable = WritableStream::from_sink(sink);

    let mut writer = writable.get_writer().unwrap();
    assert_eq!(writer.write(JsValue::from("Hello")).await.unwrap(), ());
    assert_eq!(writer.write(JsValue::from("world!")).await.unwrap(), ());
    assert_eq!(writer.close().await.unwrap(), ());
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
