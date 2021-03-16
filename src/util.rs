use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

pub(crate) async fn promise_to_void_future(promise: Promise) -> Result<(), JsValue> {
    let js_value = JsFuture::from(promise).await?;
    debug_assert!(js_value.is_undefined());
    let _ = js_value;
    Ok(())
}

pub(crate) fn clamp_to_u32(value: usize) -> u32 {
    let wrapped = value as u32;
    let overflow = value != (wrapped as usize);
    if overflow {
        u32::MAX
    } else {
        wrapped
    }
}
