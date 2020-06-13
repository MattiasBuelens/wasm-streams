use wasm_bindgen::prelude::*;

use wasm_streams::writable::*;

#[wasm_bindgen(module = "/tests/js/writable_stream.js")]
extern "C" {
    pub fn new_noop_writable_stream() -> sys::WritableStream;
    pub fn new_recording_writable_stream() -> WritableStreamAndEvents;

    #[derive(Clone, Debug)]
    pub type WritableStreamAndEvents;

    #[wasm_bindgen(method, getter)]
    pub fn stream(this: &WritableStreamAndEvents) -> sys::WritableStream;

    #[wasm_bindgen(method, getter)]
    pub fn events(this: &WritableStreamAndEvents) -> Box<[JsValue]>;
}

pub fn get_recorded_events(stream_and_events: &WritableStreamAndEvents) -> Vec<String> {
    stream_and_events
        .events()
        .into_iter()
        .map(|x| x.as_string().unwrap())
        .collect::<Vec<_>>()
}
