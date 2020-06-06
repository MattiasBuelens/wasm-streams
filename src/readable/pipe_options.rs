use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PipeOptions {
    #[wasm_bindgen(readonly, js_name = preventClose)]
    pub prevent_close: bool,
    #[wasm_bindgen(readonly, js_name = preventCancel)]
    pub prevent_cancel: bool,
    #[wasm_bindgen(readonly, js_name = preventAbort)]
    pub prevent_abort: bool,
    // TODO abort signal
    _private: (),
}

impl PipeOptions {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn prevent_close(&mut self, prevent_close: bool) -> &mut Self {
        self.prevent_close = prevent_close;
        self
    }

    pub fn prevent_cancel(&mut self, prevent_cancel: bool) -> &mut Self {
        self.prevent_cancel = prevent_cancel;
        self
    }

    pub fn prevent_abort(&mut self, prevent_abort: bool) -> &mut Self {
        self.prevent_abort = prevent_abort;
        self
    }
}
