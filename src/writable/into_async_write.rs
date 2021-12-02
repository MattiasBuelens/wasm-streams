use std::pin::Pin;
use std::task::{Context, Poll};

use futures::io::AsyncWrite;
use futures::ready;
use futures::sink::Sink;
use js_sys::Uint8Array;
use pin_project_lite::pin_project;

use crate::util::js_to_io_error;

use super::IntoSink;

pin_project! {
    #[derive(Debug)]
    #[must_use = "writers do nothing unless polled"]
    pub struct IntoAsyncWrite<'writer>
    {
        #[pin]
        sink: IntoSink<'writer>
    }
}

impl<'writer> IntoAsyncWrite<'writer> {
    #[inline]
    pub(super) fn new(sink: IntoSink<'writer>) -> Self {
        Self { sink }
    }
}

impl<'writer> AsyncWrite for IntoAsyncWrite<'writer> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let mut this = self.project();
        ready!(this.sink.as_mut().poll_ready(cx).map_err(js_to_io_error))?;
        this.sink
            .as_mut()
            .start_send(Uint8Array::from(buf).into())
            .map_err(js_to_io_error)?;
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let mut this = self.project();
        this.sink
            .as_mut()
            .poll_flush(cx)
            .map_ok(|_| ())
            .map_err(js_to_io_error)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let mut this = self.project();
        this.sink
            .as_mut()
            .poll_close(cx)
            .map_ok(|_| ())
            .map_err(js_to_io_error)
    }
}
