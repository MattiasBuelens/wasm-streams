//! ## Reading a streaming fetch response
//!
//! This example makes an HTTP request using `fetch()` from `web-sys`,
//! and then consumes the response body as a Rust `Stream`.
use futures::StreamExt;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{console, window, Response};

use wasm_streams::ReadableStream;

async fn fetch_example(url: &str) {
    // Make a fetch request
    let window = window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_str(url))
        .await
        .expect("fetch failed");
    let resp: Response = resp_value.dyn_into().unwrap();

    // Get the response's body as a ReadableStream
    let raw_body = resp.body().unwrap();
    let body = ReadableStream::from_raw(raw_body.unchecked_into());

    // Convert the ReadableStream to a Rust stream
    let mut stream = body.into_stream().unwrap();

    // Consume the stream
    while let Some(Ok(chunk)) = stream.next().await {
        console::log_1(&chunk);
    }
}
