use wasm_bindgen::JsValue;

pub const RELEASE_LOCK_REFUSED: &str = "releaseLock refused";

/// Makes `releaseLock()` throw on the given reader or writer, like Cloudflare Workers does
/// outside a request.
pub fn refuse_release_lock(target: &JsValue) {
    let refuse =
        js_sys::Function::new_no_args(&format!("throw new Error('{RELEASE_LOCK_REFUSED}')"));
    js_sys::Reflect::set(target, &JsValue::from_str("releaseLock"), &refuse).unwrap();
}
