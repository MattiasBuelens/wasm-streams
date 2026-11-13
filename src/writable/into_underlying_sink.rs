use std::panic::{AssertUnwindSafe, RefUnwindSafe, UnwindSafe};
use std::pin::Pin;

use futures_util::{Sink, SinkExt};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(private)]
pub(crate) struct IntoUnderlyingSink {
    sink: Option<Pin<Box<dyn Sink<JsValue, Error = JsValue>>>>,
}

// SAFETY: The sink is taken out of the `Option` around every fallible await in
// `write` and `close`. On panic the sink is already gone, leaving `None`, so a
// caught panic cannot expose torn state.
impl UnwindSafe for IntoUnderlyingSink {}
impl RefUnwindSafe for IntoUnderlyingSink {}

impl IntoUnderlyingSink {
    pub fn new(sink: Box<dyn Sink<JsValue, Error = JsValue>>) -> Self {
        IntoUnderlyingSink {
            sink: Some(sink.into()),
        }
    }
}

#[wasm_bindgen]
impl IntoUnderlyingSink {
    pub async fn write(&mut self, chunk: JsValue) -> Result<(), JsValue> {
        // Take the sink out before the fallible/panickable operation.
        let mut sink = AssertUnwindSafe(self.sink.take().unwrap_throw());
        AssertUnwindSafe(sink.send(chunk)).await?;
        // Success: put the sink back for reuse
        self.sink = Some(sink.0);
        Ok(())
    }

    pub async fn close(mut self) -> Result<(), JsValue> {
        let mut sink = AssertUnwindSafe(self.sink.take().unwrap_throw());
        AssertUnwindSafe(sink.close()).await
    }

    pub fn abort(self, _reason: JsValue) {
        // Drop the sink.
    }
}
