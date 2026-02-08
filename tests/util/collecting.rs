use futures_util::Sink;
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use wasm_bindgen::JsValue;

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
