use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Debug, Default)]
pub struct PipeOptions {
    prevent_close: bool,
    prevent_cancel: bool,
    prevent_abort: bool,
    // TODO abort signal
}

impl PipeOptions {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_prevent_close(&mut self, prevent_close: bool) -> &mut Self {
        self.prevent_close = prevent_close;
        self
    }

    pub fn set_prevent_cancel(&mut self, prevent_cancel: bool) -> &mut Self {
        self.prevent_cancel = prevent_cancel;
        self
    }

    pub fn set_prevent_abort(&mut self, prevent_abort: bool) -> &mut Self {
        self.prevent_abort = prevent_abort;
        self
    }
}

#[wasm_bindgen]
impl PipeOptions {
    #[wasm_bindgen(getter, js_name = preventClose)]
    pub fn prevent_close(&self) -> bool {
        self.prevent_close
    }

    #[wasm_bindgen(getter, js_name = preventCancel)]
    pub fn prevent_cancel(&self) -> bool {
        self.prevent_cancel
    }

    #[wasm_bindgen(getter, js_name = preventAbort)]
    pub fn prevent_abort(&self) -> bool {
        self.prevent_abort
    }
}
