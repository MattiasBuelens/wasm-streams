//! Tests for panic=unwind safety in wasm-streams.
//!
//! These tests verify that wasm-streams handles panics correctly when built with
//! `-Cpanic=unwind`. The key scenarios tested are:
//!
//! 1. A panic in a user's Stream/Sink/AsyncRead implementation leaves the
//!    underlying source/sink in a clean state (None), not corrupted.
//! 2. Subsequent calls after a panic fail cleanly with an error, not UB.

use futures_util::io::AsyncRead;
use futures_util::sink::Sink;
use futures_util::stream::Stream;
use std::cell::RefCell;
use std::io;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use wasm_bindgen::prelude::*;

/// A Stream that panics on the Nth poll.
pub struct PanickingStream {
    polls_before_panic: usize,
    poll_count: usize,
}

impl PanickingStream {
    pub fn new(polls_before_panic: usize) -> Self {
        Self {
            polls_before_panic,
            poll_count: 0,
        }
    }
}

impl Stream for PanickingStream {
    type Item = Result<JsValue, JsValue>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll_count += 1;
        if self.poll_count > self.polls_before_panic {
            panic!("PanickingStream: intentional panic for testing");
        }
        Poll::Ready(Some(Ok(JsValue::from(self.poll_count))))
    }
}

/// A Sink that panics on the Nth send.
pub struct PanickingSink {
    sends_before_panic: usize,
    send_count: usize,
}

impl PanickingSink {
    pub fn new(sends_before_panic: usize) -> Self {
        Self {
            sends_before_panic,
            send_count: 0,
        }
    }
}

impl Sink<JsValue> for PanickingSink {
    type Error = JsValue;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(mut self: Pin<&mut Self>, _item: JsValue) -> Result<(), Self::Error> {
        self.send_count += 1;
        if self.send_count > self.sends_before_panic {
            panic!("PanickingSink: intentional panic for testing");
        }
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

/// An AsyncRead that panics on the Nth read.
pub struct PanickingAsyncRead {
    reads_before_panic: usize,
    read_count: usize,
}

impl PanickingAsyncRead {
    pub fn new(reads_before_panic: usize) -> Self {
        Self {
            reads_before_panic,
            read_count: 0,
        }
    }
}

impl AsyncRead for PanickingAsyncRead {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        self.read_count += 1;
        if self.read_count > self.reads_before_panic {
            panic!("PanickingAsyncRead: intentional panic for testing");
        }
        // Return some bytes
        let bytes_to_write = buf.len().min(4);
        buf[..bytes_to_write].fill(0x42);
        Poll::Ready(Ok(bytes_to_write))
    }
}

/// A simple collecting sink that stores values in a RefCell<Vec>.
pub struct CollectingSink {
    collected: Rc<RefCell<Vec<JsValue>>>,
}

impl CollectingSink {
    pub fn new() -> (Self, Rc<RefCell<Vec<JsValue>>>) {
        let collected = Rc::new(RefCell::new(Vec::new()));
        (
            Self {
                collected: collected.clone(),
            },
            collected,
        )
    }
}

impl Sink<JsValue> for CollectingSink {
    type Error = JsValue;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: JsValue) -> Result<(), Self::Error> {
        self.collected.borrow_mut().push(item);
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
