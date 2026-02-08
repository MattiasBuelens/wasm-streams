//! Tests for panic=unwind safety in wasm-streams.
//!
//! These tests verify that wasm-streams handles panics correctly when built with
//! `-Cpanic=unwind`. The key scenarios tested are:
//!
//! 1. A panic in a user's Stream/Sink/AsyncRead implementation leaves the
//!    underlying source/sink in a clean state (None), not corrupted.
//! 2. Subsequent calls after a panic fail cleanly with an error, not UB.

use wasm_bindgen::prelude::*;
use crate::util::panicking::*;
use futures_util::{SinkExt, StreamExt};
use wasm_bindgen_test::*;
use wasm_streams::{ReadableStream, WritableStream};

/// Test that a ReadableStream created from a panicking Stream
/// can be read until the panic, and that the panic is properly
/// converted to a JavaScript exception.
#[wasm_bindgen_test]
async fn test_readable_stream_panic_is_caught() {
    // Create a stream that panics after 2 successful polls
    let panicking_stream = PanickingStream::new(2);
    let readable = ReadableStream::from_stream(panicking_stream);
    let mut stream = readable.into_stream();

    // First two reads should succeed
    let result1 = stream.next().await;
    assert!(result1.is_some());
    assert!(result1.unwrap().is_ok());

    let result2 = stream.next().await;
    assert!(result2.is_some());
    assert!(result2.unwrap().is_ok());

    // Third read should trigger the panic, which should be caught
    // and converted to an error by wasm-bindgen
    let result3 = stream.next().await;
    // With panic=unwind, this should be an error, not a crash
    assert!(result3.is_some());
    let inner = result3.unwrap();
    assert!(
        inner.is_err(),
        "Expected error from panic, got: {:?}",
        inner
    );

    // Stream should be closed
    let result4 = stream.next().await;
    assert!(
        result4.is_none(),
        "Fourth read should be closed, got {:?}",
        result4
    );
}

/// Test that a WritableStream created from a panicking Sink
/// handles the panic correctly.
#[wasm_bindgen_test]
async fn test_writable_stream_panic_is_caught() {
    // Create a sink that panics after 2 successful sends
    let panicking_sink = PanickingSink::new(2);
    let writable = WritableStream::from_sink(panicking_sink);
    let mut sink = writable.into_sink();

    // First two writes should succeed
    let result1 = sink.send(JsValue::from(1)).await;
    assert!(result1.is_ok(), "First write failed: {:?}", result1);

    let result2 = sink.send(JsValue::from(2)).await;
    assert!(result2.is_ok(), "Second write failed: {:?}", result2);

    // Third write should trigger the panic, which should be caught
    // and converted to an error by wasm-bindgen
    let result3 = sink.send(JsValue::from(3)).await;
    // With panic=unwind, this should be an error, not a crash
    assert!(
        result3.is_err(),
        "Expected error from panic, got: {:?}",
        result3
    );
}

/// Test that after a panic, the underlying sink is in a clean state (None),
/// not corrupted. This verifies the take-and-replace pattern works correctly.
/// Subsequent writes after a panic should return an error.
#[wasm_bindgen_test]
async fn test_panic_leaves_clean_state() {
    // Create a sink that panics on first send
    let panicking_sink = PanickingSink::new(0);
    let writable = WritableStream::from_sink(panicking_sink);
    let mut sink = writable.into_sink();

    // First write triggers panic, which should be caught and converted to an error.
    // The key test here is that we get a clean error (not a crash, not UB,
    // not corrupted state being reused).
    let result1 = sink.send(JsValue::from(1)).await;
    assert!(result1.is_err(), "Expected error from panic on first write");

    // Subsequent writes after the panic should also return an error,
    // since the stream is now in an errored state.
    let result2 = sink.send(JsValue::from(2)).await;
    assert!(
        result2.is_err(),
        "Expected error on write after panic, got: {:?}",
        result2
    );
}

/// Test that a ReadableStream created from a panicking AsyncRead
/// handles the panic correctly.
#[wasm_bindgen_test]
async fn test_async_read_panic_is_caught() {
    use futures_util::io::AsyncReadExt;

    // Create an AsyncRead that panics after 2 successful reads
    let panicking_reader = PanickingAsyncRead::new(2);
    let readable = ReadableStream::from_async_read(panicking_reader, 4);
    let mut reader = readable.into_async_read();

    // First two reads should succeed
    let mut buf = [0u8; 4];

    let result1 = reader.read(&mut buf).await;
    assert!(result1.is_ok(), "First read failed: {:?}", result1);

    let result2 = reader.read(&mut buf).await;
    assert!(result2.is_ok(), "Second read failed: {:?}", result2);

    // Third read should trigger the panic, which should be caught
    // and converted to an error by wasm-bindgen
    let result3 = reader.read(&mut buf).await;
    assert!(
        result3.is_err(),
        "Expected error from panic, got: {:?}",
        result3
    );

    // AsyncRead should be closed
    let result4 = reader.read(&mut buf).await;
    assert!(
        matches!(result4, Ok(0)),
        "Fourth read should be Ok(0), got: {:?}",
        result4
    );
}

/// Basic sanity test that normal (non-panicking) streams work correctly.
#[wasm_bindgen_test]
async fn test_normal_stream_works() {
    let items = vec![
        Ok(JsValue::from(1)),
        Ok(JsValue::from(2)),
        Ok(JsValue::from(3)),
    ];
    let stream = futures_util::stream::iter(items);
    let readable = ReadableStream::from_stream(stream);
    let mut rs = readable.into_stream();

    let mut count = 0;
    while let Some(result) = rs.next().await {
        assert!(result.is_ok());
        count += 1;
    }
    assert_eq!(count, 3);
}

/// Basic sanity test that normal (non-panicking) sinks work correctly.
#[wasm_bindgen_test]
async fn test_normal_sink_works() {
    let (sink, collected) = CollectingSink::new();
    let writable = WritableStream::from_sink(sink);
    let mut ws = writable.into_sink();

    ws.send(JsValue::from(1)).await.unwrap();
    ws.send(JsValue::from(2)).await.unwrap();
    ws.send(JsValue::from(3)).await.unwrap();

    // Close the stream
    ws.close().await.unwrap();

    assert_eq!(collected.borrow().len(), 3);
}
