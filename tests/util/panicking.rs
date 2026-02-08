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
