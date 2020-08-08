# wasm-streams

[![Build Status](https://img.shields.io/github/workflow/status/MattiasBuelens/wasm-streams/CI.svg)](https://github.com/MattiasBuelens/wasm-streams)
[![Crates.io Version](https://img.shields.io/crates/v/wasm-streams.svg)](https://crates.io/crates/wasm-streams)
[![Docs.rs](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/wasm-streams)

This crate bridges the gap between [web streams](https://developer.mozilla.org/en-US/docs/Web/API/Streams_API) 
and [Rust streams from the futures crate](https://docs.rs/futures/latest/futures/stream).

It provides Rust APIs for interacting with a JavaScript `ReadableStream`, `WritableStream` or `TransformStream`.
It also allows converting between a `ReadableStream` and a Rust `Stream`, 
as well as between a `WritableStream` and a Rust `Sink`.

See the [API documentation](https://docs.rs/wasm-streams) for more information,
or check out the [examples](https://github.com/MattiasBuelens/wasm-streams/tree/master/examples).
