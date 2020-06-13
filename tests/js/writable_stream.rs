use wasm_bindgen::prelude::*;

use wasm_streams::writable::*;

#[wasm_bindgen(module = "/tests/js/writable_stream.js")]
extern "C" {
    pub fn new_noop_writable_stream() -> sys::WritableStream;
    fn new_recording_writable_stream() -> WritableStreamAndEvents;

    #[derive(Clone, Debug)]
    type WritableStreamAndEvents;

    #[wasm_bindgen(method, getter)]
    fn stream(this: &WritableStreamAndEvents) -> sys::WritableStream;

    #[wasm_bindgen(method, getter)]
    fn events(this: &WritableStreamAndEvents) -> Box<[JsValue]>;
}

pub struct RecordingWritableStream {
    raw: WritableStreamAndEvents,
}

impl RecordingWritableStream {
    pub fn new() -> Self {
        Self {
            raw: new_recording_writable_stream(),
        }
    }

    pub fn stream(&self) -> sys::WritableStream {
        self.raw.stream()
    }

    pub fn events(&self) -> Vec<String> {
        self.raw
            .events()
            .into_iter()
            .map(|x| x.as_string().unwrap())
            .collect::<Vec<_>>()
    }
}
