use super::sys;
use web_sys::AbortSignal;

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

    /// Convert this to a raw [`PipeOptions`](sys::PipeOptions) object.
    pub fn as_raw(&self) -> sys::PipeOptions {
        sys::PipeOptions::new(
            self.prevent_close,
            self.prevent_cancel,
            self.prevent_abort,
            self.signal.clone(),
        )
    }
}
