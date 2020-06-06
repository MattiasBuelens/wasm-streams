//! ## Reading a streaming fetch response
//!
//! This example makes an HTTP request using `fetch()` from `web-sys`,
//! and then consumes the response body as a Rust `Stream`.

use futures::StreamExt;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{console, window, Response};

use wasm_streams::ReadableStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Make a fetch request
    let url = "http://example.com/";
    let window = window().unwrap_throw();
    let resp_value = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|_| "fetch failed")?;
    let resp: Response = resp_value.dyn_into().unwrap_throw();

    // Get the response's body as a ReadableStream
    let raw_body = resp.body().unwrap_throw();
    let body = ReadableStream::from_raw(raw_body.dyn_into().unwrap_throw());

    // Convert the ReadableStream to a Rust stream
    let mut stream = body.into_stream();

    // Consume the stream
    while let Some(Ok(chunk)) = stream.next().await {
        console::log_1(&chunk);
    }

    Ok(())
}
