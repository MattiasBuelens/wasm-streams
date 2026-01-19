//! Minimal reproduction: IntoSink silently swallows writes after an error
//!
//! Bug: When the underlying sink errors, subsequent writes to IntoSink
//! return Ok(()) instead of an error. The stream should be in an errored
//! state and reject all further operations.
//!
//! Run with: wasm-pack test --node

use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::sink::Sink;
use futures_util::SinkExt;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use wasm_streams::WritableStream;

wasm_bindgen_test_configure!(run_in_node_experimental);

/// A Sink that always errors on the first write.
struct FailingSink {
    failed: bool,
}

impl FailingSink {
    fn new() -> Self {
        Self { failed: false }
    }
}

impl Sink<JsValue> for FailingSink {
    type Error = JsValue;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(mut self: Pin<&mut Self>, _item: JsValue) -> Result<(), Self::Error> {
        if !self.failed {
            self.failed = true;
            // Return an error on first write
            Err(JsValue::from_str("intentional error"))
        } else {
            Ok(())
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.failed {
            Poll::Ready(Err(JsValue::from_str("sink has failed")))
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

#[wasm_bindgen_test]
async fn test_into_sink_errors_after_failure() {
    let failing_sink = FailingSink::new();
    let writable = WritableStream::from_sink(failing_sink);
    let mut sink = writable.into_sink();

    // First write should fail
    let result1 = sink.send(JsValue::from(1)).await;
    assert!(result1.is_err(), "First write should fail");

    // BUG: Second write returns Ok(()) instead of an error!
    // After an error, the stream should be in an errored state and
    // reject all subsequent operations.
    let result2 = sink.send(JsValue::from(2)).await;
    assert!(
        result2.is_err(),
        "Second write should fail because stream is errored, but got Ok(())"
    );
}
