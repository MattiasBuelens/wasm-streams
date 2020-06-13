use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

use wasm_streams::readable::sys as readable_sys;
use wasm_streams::*;

#[wasm_bindgen(module = "/tests/readable_stream.js")]
extern "C" {
    fn new_readable_stream_from_array(chunks: Box<[JsValue]>) -> readable_sys::ReadableStream;
}

#[wasm_bindgen_test]
async fn test_pipe_js_to_rust() {
    let chunks = vec![JsValue::from("Hello"), JsValue::from("world!")];
    let mut readable = ReadableStream::from_raw(new_readable_stream_from_array(
        chunks.clone().into_boxed_slice(),
    ));

    let (sink, stream) = mpsc::unbounded::<JsValue>();
    let sink = sink.sink_map_err(|_| JsValue::from_str("cannot happen"));
    let mut writable = WritableStream::from_sink(sink);

    readable.pipe_to(&mut writable).await.unwrap();

    // All chunks must be sent to sink
    let output = stream.collect::<Vec<_>>().await;
    assert_eq!(output, chunks);

    // Both streams must be closed
    readable.get_reader().closed().await.unwrap();
    writable.get_writer().closed().await.unwrap();
}
