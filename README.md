# jc-libavif-sys

`jc-libavif-sys` is a raw Rust FFI crate for `libavif`.

It is designed to be used directly from GitHub. By default it tries to fetch a verified prebuilt native archive for the current target. If no matching prebuilt is available, it falls back to downloading the pinned upstream `libavif` and `libaom` source archives and building them with the system `cmake` executable.

## Features

- Raw Rust bindings for the public `libavif` C API, generated from upstream `avif.h`
- Coverage for the parts of `libavif` needed for modern HDR workflows, including gain maps, opaque item properties, color signaling, and encode/decode APIs
- No system `libavif` or `libaom` installation required
- Prebuilt native artifacts by default for supported targets, with automatic source-build fallback when needed
- Pinned upstream versions of `libavif` and `libaom`, with a documented maintainer flow to upgrade and regenerate bindings
- Integration tests that exercise HDR encode/decode paths, including HDR10 and higher bit depths, plus real-world HDR10 AVIF fixtures

## Requirements

- Rust 1.85 or newer
- for source fallback: `cmake` on `PATH` and a working C/C++ toolchain for the host platform

`build.rs` intentionally shells out to `cmake` instead of trying to replicate upstream native build logic in Rust.

## Supported targets

The minimum supported target set for this repository is:

- `aarch64-apple-darwin`
- `aarch64-unknown-linux-gnu`
- `x86_64-unknown-linux-gnu`

## Using from GitHub

Use a tag when you want a stable dependency:

```toml
[dependencies]
jc-libavif-sys = { git = "https://github.com/jcayzac/jc-libavif-sys", tag = "v1.3.0" }
```

Use a revision when you need an exact commit:

```toml
[dependencies]
jc-libavif-sys = { git = "https://github.com/jcayzac/jc-libavif-sys", rev = "0123456789abcdef" }
```

No system `libavif` or `libaom` installation is required.

## Build selection

By default, the crate prefers a prebuilt native archive for the current target. If that path fails, it falls back to downloading the pinned upstream source tarballs and building them locally with CMake.

The following environment variables control that behavior:

- `JC_LIBAVIF_SYS_USE_PREBUILT=1`
  Prefer a prebuilt native archive. This is effectively the default behavior, but the variable is still accepted for explicitness.
- `JC_LIBAVIF_SYS_PREBUILT_ONLY=1`
  Require a prebuilt native archive and disable source download and local compilation.
- `JC_LIBAVIF_SYS_NO_PREBUILT=1`
  Disable prebuilt archives and require the downloaded-source CMake build.
- `JC_LIBAVIF_SYS_PREBUILT_BASE_URL=<url>`
  Override the base URL used to fetch prebuilt archives. The build expects files named `jc-libavif-sys-native-<target>.tar.gz` and matching `.sha256` sidecars under this base URL.
- `JC_LIBAVIF_SYS_PREBUILT_TAG=<tag>`
  When `JC_LIBAVIF_SYS_PREBUILT_BASE_URL` is not set, override the release tag used to construct the default GitHub release download URL.

If neither `JC_LIBAVIF_SYS_PREBUILT_BASE_URL` nor Cargo package `repository` metadata is available, the default prebuilt attempt is skipped and the build falls back to source compilation.

Conflict rules:

- `JC_LIBAVIF_SYS_PREBUILT_ONLY=1` cannot be combined with `JC_LIBAVIF_SYS_NO_PREBUILT=1`
- `JC_LIBAVIF_SYS_USE_PREBUILT=1` cannot be combined with `JC_LIBAVIF_SYS_NO_PREBUILT=1`

The prebuilt path is implemented in Rust inside `build.rs`; it does not require external download, archive, or checksum CLI tools.

## Binding scope

Bindings are generated from upstream `avif.h`, which includes public APIs for:

- encode and decode
- gain maps
- opaque item properties
- color signaling and light-level metadata
- ICC, Exif, XMP, transforms, grids, and sequences

The crate keeps the dependency surface minimal and only uses `libaom` as the AV1 codec backend.

## Test fixtures

The repository vendors two small real-world HDR10 AVIF decode fixtures from the AOMedia `av1-avif` Netflix corpus:

- `hdr_cosmos01000_cicp9-16-9_yuv420_limited_qp10.avif`
- `hdr_cosmos01000_cicp9-16-9_yuv444_full_qp10.avif`

They are used as decode/interoperability fixtures only. The larger PNG sidecars are intentionally not committed.

## Licensing

The `license = "MIT OR Apache-2.0"` declaration in `Cargo.toml` applies to the original code in this repository, such as the Rust crate source, build glue, tests, CI configuration, and maintainer tooling.

That dual license does not relicense upstream third-party code.

The original repository code is dual-licensed under:

- Apache License 2.0
- MIT license

See `LICENSE-APACHE` and `LICENSE-MIT`.

The downloaded or prebuilt third-party native code keeps its own upstream licenses and notices:

- `libavif`: BSD-2-Clause-style license
- `libaom`: BSD-2-Clause-style license plus `PATENTS`

See `THIRD_PARTY.md` for details.

## Maintainers

Build, test, and upgrade instructions live in `MAINTAINING.md`.
