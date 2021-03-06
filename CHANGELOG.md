# Changelog

## v0.2.0 (2021-06-22)

* Add support for readable byte streams ([#6](https://github.com/MattiasBuelens/wasm-streams/pull/6))
    * Add `ReadableStream::(try_)get_byob_reader` to acquire
      a [BYOB reader](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamBYOBReader).
    * Add `ReadableStream::from_async_read` to turn
      an [`AsyncRead`](https://docs.rs/futures/0.3.15/futures/io/trait.AsyncRead.html)
      into a readable byte stream.
    * Add `ReadableStream::(try_)into_async_read` to turn a readable byte stream into
      an [`AsyncRead`](https://docs.rs/futures/0.3.15/futures/io/trait.AsyncRead.html).
* Improve error handling and drop behavior of `ReadableStream::from_stream()`

## v0.1.2 (2020-10-31)

* Include license files in repository ([#5](https://github.com/MattiasBuelens/wasm-streams/issues/5))

## v0.1.1 (2020-08-08)

* Specify TypeScript type for raw streams ([#1](https://github.com/MattiasBuelens/wasm-streams/pull/1))

## v0.1.0 (2020-06-15)

First release! 🎉
