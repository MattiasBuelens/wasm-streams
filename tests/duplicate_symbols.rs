//! Regression test for <https://github.com/cloudflare/workers-rs/issues/1026>:
//! linking two semver-incompatible versions of `wasm-streams` into the same
//! `cdylib` must not produce duplicate `wasm-bindgen` symbols.
//!
//! The actual check lives in the `duplicate-symbols` crate alongside this
//! one; building it for `wasm32-unknown-unknown` *is* the test; a duplicate
//! symbol turns into a `rust-lld` link error. This just drives that build
//! from `cargo test`, so it runs on CI without a dedicated job.
//!
//! This can only run natively, since it shells out to `cargo` itself.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

#[test]
fn duplicate_symbols() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("duplicate-symbols");
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let status = Command::new(cargo)
        .arg("build")
        .arg("--target=wasm32-unknown-unknown")
        .current_dir(&manifest_dir)
        // Build with a clean, default environment: the point is to check
        // whether `wasm-streams` itself introduces duplicate symbols, not
        // to reproduce whatever unstable flags the outer build might use.
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .status()
        .expect("failed to run `cargo build` for the `duplicate-symbols` crate");
    assert!(
        status.success(),
        "building the `duplicate-symbols` crate failed; this usually means two \
         versions of `wasm-streams` export conflicting wasm-bindgen symbols, \
         see https://github.com/cloudflare/workers-rs/issues/1026"
    );
}
