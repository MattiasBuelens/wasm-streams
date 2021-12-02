use std::pin::Pin;
use std::task::{Context, Poll};

use futures::io::AsyncWrite;
use futures::ready;
use futures::sink::Sink;
use pin_project_lite::pin_project;

pin_project! {
    #[derive(Debug)]
    #[must_use = "writers do nothing unless polled"]
    pub struct IntoAsyncWrite<Si>
    {
        #[pin]
        sink: Si
    }
}

impl<Si> IntoAsyncWrite<Si> {
    #[inline]
    pub(super) fn new(sink: Si) -> Self {
        Self { sink }
    }
}

impl<Si> AsyncWrite for IntoAsyncWrite<Si>
where
    Si: Sink<Box<[u8]>, Error = std::io::Error>,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let mut this = self.project();
        ready!(this.sink.as_mut().poll_ready(cx))?;
        this.sink.as_mut().start_send(buf.into())?; // buf.into() makes a copy
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let mut this = self.project();
        this.sink.as_mut().poll_flush(cx).map_ok(|_| ())
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let mut this = self.project();
        this.sink.as_mut().poll_close(cx).map_ok(|_| ())
    }
}
