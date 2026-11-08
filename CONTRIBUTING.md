# Testing

The tests use [wasm-pack](https://drager.github.io/wasm-pack/).
See the [wasm-bindgen guide](https://wasm-bindgen.github.io/wasm-bindgen/wasm-bindgen-test/usage.html) for more information.

We run the tests in Node.js, Chrome and Firefox:
```
wasm-pack test --node
WASM_BINDGEN_USE_BROWSER=1 wasm-pack test --headless --chrome
WASM_BINDGEN_USE_BROWSER=1 wasm-pack test --headless --firefox
```

When debugging the browser tests, remove the `--headless` flag:
```
WASM_BINDGEN_USE_BROWSER=1 wasm-pack test --chrome
```

# Changelog

We use [Changie](https://changie.dev/guide/quick_start/) to manage release notes.
Install [v1.26.0](https://github.com/miniscruff/changie/releases/tag/v1.26.0), run
`changie new`, and commit the generated file in `.changes/unreleased/` with your
change. Write the entry as Markdown, including relevant PR or issue links.
`CHANGELOG.md` is generated; add new entries through fragments.

PRs must add a new fragment unless a maintainer applies `skip-changelog`.
Dependabot and generated release PRs are exempt. See the [Changie CLI docs](https://changie.dev/cli/changie_new/)
for editing options and the [configuration reference](https://changie.dev/config/)
for formatting details.

# Preparing a release

1. In GitHub Actions, run **Prepare release** on `main`, choosing a `major`,
   `minor`, or `patch` bump.
1. Review the generated draft PR, which updates the changelog and the crate
   version in `Cargo.toml`. Mark it **Ready for review**, wait for CI to pass,
   and merge it.
1. Merging the release PR runs **Draft release**, which drafts a GitHub Release
   for the new `v<version>` tag using the notes in `.changes/v<version>.md`.
1. Review and publish the draft release. This creates the tag, which triggers
   publishing to crates.io.
