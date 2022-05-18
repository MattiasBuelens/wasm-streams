use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

use futures_util::{Sink, Stream};
use pin_project::pin_project;

#[pin_project]
#[derive(Debug)]
pub struct SimpleChannel<T> {
    queue: VecDeque<T>,
    waker: Option<Waker>,
    closed: bool,
}

impl<T> SimpleChannel<T> {
    pub fn new() -> Self {
        SimpleChannel {
            queue: VecDeque::new(),
            waker: None,
            closed: false,
        }
    }
}

impl<T> Stream for SimpleChannel<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this.queue.pop_front() {
            Some(item) => Poll::Ready(Some(item)),
            None if *this.closed => Poll::Ready(None),
            None => {
                *this.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.queue.len(), None)
    }
}

impl<T> Sink<T> for SimpleChannel<T> {
    type Error = ();

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        let this = self.project();
        this.queue.push_back(item);
        if let Some(waker) = this.waker.take() {
            waker.wake();
        }
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        *this.closed = true;
        if let Some(waker) = this.waker.take() {
            waker.wake();
        }
        Poll::Ready(Ok(()))
    }
}
