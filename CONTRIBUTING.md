# Testing

The tests use [wasm-pack](https://rustwasm.github.io/wasm-pack/).
See the [wasm-bindgen guide](https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/usage.html) for more information.

We only run the tests on Chrome, since it's currently the only browser that supports the full streams API.
Run the tests with:
```
WASM_BINDGEN_USE_BROWSER=1 wasm-pack test --headless --chrome
```

When debugging the tests, remove the `--headless` flag:
```
WASM_BINDGEN_USE_BROWSER=1 wasm-pack test --chrome
```
