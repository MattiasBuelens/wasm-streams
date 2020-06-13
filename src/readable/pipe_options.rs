use super::sys;

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

    pub fn as_raw(&self) -> sys::PipeOptions {
        sys::PipeOptions::new(self.prevent_close, self.prevent_cancel, self.prevent_abort)
    }
}
