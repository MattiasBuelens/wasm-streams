use std::time::Duration;

use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = setTimeout)]
    fn set_timeout(handler: &::js_sys::Function, timeout: i32) -> JsValue;
}

/// Resolves after the given duration using the global `setTimeout`.
///
/// Avoids depending on `gloo-timers`, which currently fails to compile against
/// recent `wasm-bindgen` versions when `panic=unwind` is enabled (see
/// rustwasm/gloo#562).
pub async fn sleep(duration: Duration) {
    let millis = duration.as_millis() as i32;
    let promise = Promise::new(&mut |resolve, _reject| {
        set_timeout(&resolve, millis);
    });
    let _ = JsFuture::from(promise).await;
}
