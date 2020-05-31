# wasm-streams

This crate bridges the gap between [web streams][web-streams] and [Rust streams from the futures crate][rust-futures].
It provides Rust APIs for interacting with a JavaScript `ReadableStream`, `WritableStream` or `TransformStream`.
It also allows converting between a `ReadableStream` and a Rust `Stream`, 
as well as between a `WritableStream` and a Rust `Sink`.

[web-streams]: https://developer.mozilla.org/en-US/docs/Web/API/Streams_API
[rust-futures]: https://docs.rs/futures/
