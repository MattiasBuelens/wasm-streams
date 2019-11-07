use core::fmt;
use core::pin::Pin;

use futures::future::Future;
use futures::ready;
use futures::stream::{FusedStream, Stream};
use futures::task::{Context, Poll};
use pin_utils::{unsafe_pinned, unsafe_unpinned};

pub fn unfold<T, F, Fut, Item>(init: T, f: F) -> Unfold<T, F, Fut>
    where F: FnMut(T) -> Fut,
          Fut: Future<Output=Option<(Item, T)>>,
{
    Unfold {
        f,
        state: Some(init),
        fut: None,
    }
}

#[must_use = "streams do nothing unless polled"]
pub struct Unfold<T, F, Fut> {
    f: F,
    state: Option<T>,
    fut: Option<Fut>,
}

impl<T, F, Fut: Unpin> Unpin for Unfold<T, F, Fut> {}

impl<T, F, Fut> fmt::Debug for Unfold<T, F, Fut>
    where
        T: fmt::Debug,
        Fut: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Unfold")
            .field("state", &self.state)
            .field("fut", &self.fut)
            .finish()
    }
}

impl<T, F, Fut> Unfold<T, F, Fut> {
    unsafe_unpinned!(f: F);
    unsafe_unpinned!(state: Option<T>);
    unsafe_pinned!(fut: Option<Fut>);
}

impl<T, F, Fut, Item> FusedStream for Unfold<T, F, Fut>
    where F: FnMut(T) -> Fut,
          Fut: Future<Output=Option<(Item, T)>>,
{
    fn is_terminated(&self) -> bool {
        self.state.is_none() && self.fut.is_none()
    }
}

impl<T, F, Fut, Item> Stream for Unfold<T, F, Fut>
    where F: FnMut(T) -> Fut,
          Fut: Future<Output=Option<(Item, T)>>,
{
    type Item = Item;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if let Some(state) = self.as_mut().state().take() {
            let fut = (self.as_mut().f())(state);
            self.as_mut().fut().set(Some(fut));
        }

        let step = ready!(self.as_mut().fut().as_pin_mut().unwrap().poll(cx));
        self.as_mut().fut().set(None);

        if let Some((item, next_state)) = step {
            *self.as_mut().state() = Some(next_state);
            Poll::Ready(Some(item))
        } else {
            Poll::Ready(None)
        }
    }
}
