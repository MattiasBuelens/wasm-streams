use crate::readable::ReadableStream;
use crate::writable::WritableStream;

pub mod sys;

pub struct TransformStream {
    raw: sys::TransformStream,
}

impl TransformStream {
    #[inline]
    pub fn as_raw(&self) -> &sys::TransformStream {
        &self.raw
    }

    #[inline]
    pub fn into_raw(self) -> sys::TransformStream {
        self.raw
    }

    pub fn readable(&self) -> ReadableStream {
        ReadableStream::from(self.raw.readable())
    }

    pub fn writable(&self) -> WritableStream {
        WritableStream::from(self.raw.writable())
    }
}

impl From<sys::TransformStream> for TransformStream {
    fn from(raw: sys::TransformStream) -> TransformStream {
        TransformStream { raw }
    }
}
