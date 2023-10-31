use wasm_bindgen::JsCast;
use web_sys::AbortSignal;

use super::sys;

/// Options for [`pipe_to_with_options`](super::ReadableStream::pipe_to_with_options).
#[derive(Clone, Debug, Default)]
pub struct PipeOptions {
    prevent_close: bool,
    prevent_cancel: bool,
    prevent_abort: bool,
    signal: Option<AbortSignal>,
}

impl PipeOptions {
    /// Creates a blank new set of pipe options.
    ///
    /// Equivalent to [`PipeOptions::default`](Default::default).
    pub fn new() -> Self {
        Default::default()
    }

    /// Creates a set of pipe options from a raw [`PipeOptions`](sys::PipeOptions) object.
    pub fn from_raw(raw: sys::PipeOptions) -> Self {
        let raw: &sys::StreamPipeOptionsExt = raw.unchecked_ref();
        Self {
            prevent_close: raw.prevent_close(),
            prevent_cancel: raw.prevent_cancel(),
            prevent_abort: raw.prevent_abort(),
            signal: raw.signal(),
        }
    }

    /// Convert this to a raw [`PipeOptions`](sys::PipeOptions) object.
    pub fn into_raw(self) -> sys::PipeOptions {
        let options = sys::StreamPipeOptionsExt::new();
        options.set_prevent_close(self.prevent_close);
        options.set_prevent_cancel(self.prevent_cancel);
        options.set_prevent_abort(self.prevent_abort);
        if let signal @ Some(_) = self.signal {
            options.set_signal(signal);
        }
        options.unchecked_into()
    }

    /// Sets whether the destination writable stream should be closed
    /// when the source readable stream closes.
    pub fn prevent_close(&mut self, prevent_close: bool) -> &mut Self {
        self.prevent_close = prevent_close;
        self
    }

    /// Sets whether the source readable stream should be [canceled](https://streams.spec.whatwg.org/#cancel-a-readable-stream)
    /// when the destination writable stream errors.
    pub fn prevent_cancel(&mut self, prevent_cancel: bool) -> &mut Self {
        self.prevent_cancel = prevent_cancel;
        self
    }

    /// Sets whether the destination writable stream should be [aborted](https://streams.spec.whatwg.org/#abort-a-writable-stream)
    /// when the source readable stream errors.
    pub fn prevent_abort(&mut self, prevent_abort: bool) -> &mut Self {
        self.prevent_abort = prevent_abort;
        self
    }

    /// Sets an abort signal to abort the ongoing pipe operation.
    /// When the signal is aborted, the source readable stream will be canceled
    /// and the destination writable stream will be aborted
    /// unless the respective options [`prevent_cancel`](Self::prevent_cancel)
    /// or [`prevent_abort`](Self::prevent_abort) are set.
    pub fn signal(&mut self, signal: AbortSignal) -> &mut Self {
        self.signal = Some(signal);
        self
    }
}
