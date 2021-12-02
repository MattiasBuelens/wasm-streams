use std::pin::Pin;
use std::task::{Context, Poll};

use futures::io::AsyncWrite;
use futures::ready;
use futures::sink::SinkExt;
use js_sys::Uint8Array;

use crate::util::js_to_io_error;

use super::IntoSink;

#[derive(Debug)]
#[must_use = "writers do nothing unless polled"]
pub struct IntoAsyncWrite<'writer> {
    sink: IntoSink<'writer>,
}

impl<'writer> IntoAsyncWrite<'writer> {
    #[inline]
    pub(super) fn new(sink: IntoSink<'writer>) -> Self {
        Self { sink }
    }
}

impl<'writer> AsyncWrite for IntoAsyncWrite<'writer> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        ready!(self
            .as_mut()
            .sink
            .poll_ready_unpin(cx)
            .map_err(js_to_io_error))?;
        self.as_mut()
            .sink
            .start_send_unpin(Uint8Array::from(buf).into())
            .map_err(js_to_io_error)?;
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.as_mut()
            .sink
            .poll_flush_unpin(cx)
            .map_ok(|_| ())
            .map_err(js_to_io_error)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.as_mut()
            .sink
            .poll_close_unpin(cx)
            .map_ok(|_| ())
            .map_err(js_to_io_error)
    }
}
