#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

mod pipe;
mod readable_stream;
mod transform_stream;
mod writable_stream;
