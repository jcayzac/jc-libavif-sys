# Maintaining `jc-libavif-sys`

## Prerequisites

- Rust 1.85 or newer
- `curl` and `tar` for `xtask` upgrade, bindings regeneration, and `scripts/package-prebuilt.sh`
- for source builds: `cmake` on `PATH` and a working C/C++ toolchain
- either `sha256sum` or `shasum` for `scripts/package-prebuilt.sh`
- `rustfmt` and `clippy`

`xtask generate-bindings` also needs `libclang` available to `bindgen`.

On `x86_64` builds, `build.rs` automatically falls back to `-DAOM_TARGET_CPU=generic` when neither `nasm` nor `yasm` is available, so those assemblers are optional rather than required.

## Common commands

Format the workspace:

```bash
cargo fmt --all
```

Run tests:

```bash
cargo test --workspace --all-targets
```

Run lints:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Check that committed bindings are current:

```bash
cargo run -p xtask -- generate-bindings --check
```

Create and push the release tag that matches `Cargo.toml`:

```bash
cargo run -p xtask -- release
```

## Build mode environment variables

The crate build supports these environment variables:

- `JC_LIBAVIF_SYS_PREBUILT_ONLY=1`
  Require a prebuilt archive and disable local source builds.
- `JC_LIBAVIF_SYS_NO_PREBUILT=1`
  Disable prebuilt archives and require a local source build.
- `JC_LIBAVIF_SYS_PREBUILT_BASE_URL=<url>`
  Base URL to fetch prebuilt archives from.

The prebuilt archive naming convention is:

```text
jc-libavif-sys-native-<target-triple>.tar.gz
jc-libavif-sys-native-<target-triple>.tar.gz.sha256
```

## Updating upstream library versions

Current pinned versions live in `upstream/versions.toml`.

The crate release version should track the pinned `libavif` version. For example, when `upstream/versions.toml` says `libavif = "v1.3.0"`, the crate version should normally be `1.3.0`.

Cargo package versions are three-part semver, so a fourth numeric component such as `1.3.0.1` is not valid. If you need a crate-only follow-up release without changing the pinned `libavif` version, use a normal Cargo patch bump such as `1.3.1` and leave the upstream pin unchanged.

Upgrade both `libavif` and `libaom` to the latest stable releases and regenerate bindings:

```bash
cargo run -p xtask -- upgrade --libavif latest --libaom latest
```

Upgrade only one library:

```bash
cargo run -p xtask -- upgrade --libavif v1.3.0
cargo run -p xtask -- upgrade --libaom v3.13.1
```

The upgrade command:

- resolves the requested versions
- updates `upstream/versions.toml`
- regenerates `src/bindings.rs`

## Regenerating bindings only

If you edited the version pins manually:

```bash
cargo run -p xtask -- generate-bindings
```

`xtask` formats generated bindings with `rustfmt` before writing them, so `generate-bindings --check` is stable in CI.

## Test strategy

The test suite covers two paths:

- synthetic encode/decode round-trips for 10-bit HDR10 PQ and 12-bit HLG-style content
- decode of two real-world HDR10 AVIF fixtures from the AOMedia Netflix corpus

The fixture source is documented in `tests/fixtures/README.md`.

## Before merging changes

Run all of the following:

```bash
cargo fmt --all
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p xtask -- generate-bindings --check
```

## Publishing prebuilts

The repository includes a GitHub workflow that builds prebuilt native archives for the supported targets.

The prebuilt workflow and `build.rs` assume the GitHub release tag is exactly `v{package.version}`.

Use `cargo run -p xtask -- release` to:

- refuse to run if the working tree is dirty
- create the `v{package.version}` git tag
- push that tag to the `origin` remote

Examples:

- `v1.3.0`
- `v1.3.0-rc1`

Each archive contains the installed native output layout produced by the crate build:

- `include/`
- `lib/`
- `licenses/`

Each prebuilt archive is published alongside a `.sha256` sidecar and can be used by clients through the prebuilt build-mode environment variables above.
