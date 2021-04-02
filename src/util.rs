use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

pub(crate) async fn promise_to_void_future(promise: Promise) -> Result<(), JsValue> {
    let js_value = JsFuture::from(promise).await?;
    debug_assert!(js_value.is_undefined());
    let _ = js_value;
    Ok(())
}
