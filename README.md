# jc-libavif-sys

`jc-libavif-sys` is a raw Rust FFI crate for `libavif`.

By default it tries to fetch a verified prebuilt native archive for the current target from the GitHub release that matches the crate version. If no matching prebuilt is available, it falls back to downloading the pinned upstream `libavif` and `libaom` source archives and building them with the system `cmake` executable.

## Features

- Raw Rust bindings for the public `libavif` C API, generated from upstream `avif.h`
- Coverage for the parts of `libavif` needed for modern HDR workflows, including gain maps, opaque item properties, color signaling, and encode/decode APIs
- No system `libavif` or `libaom` installation required
- Prebuilt native artifacts by default for supported targets, with automatic source-build fallback when needed
- Pinned upstream versions of `libavif` and `libaom`, with a documented maintainer flow to upgrade and regenerate bindings
- Integration tests that exercise HDR encode/decode paths, including HDR10 and higher bit depths, plus real-world HDR10 AVIF fixtures

## Requirements

- Rust 1.85 or newer
- When compiling from source: `cmake` on `PATH` and a working C/C++ toolchain for the host platform

`build.rs` intentionally shells out to `cmake` instead of trying to replicate upstream native build logic in Rust.

## Supported targets

The minimum supported target set for this repository is:

- `aarch64-apple-darwin`
- `aarch64-unknown-linux-gnu`
- `x86_64-unknown-linux-gnu`

## Build selection

By default, the crate prefers a prebuilt native archive for the current target. If that path fails, it falls back to downloading the pinned upstream source tarballs and building them locally with CMake.

The following environment variables control that behavior:

- `JC_LIBAVIF_SYS_PREBUILT_ONLY=1`
  Require a prebuilt native archive and disable source download and local compilation.
- `JC_LIBAVIF_SYS_NO_PREBUILT=1`
  Disable prebuilt archives and require the downloaded-source CMake build.

Conflict rules:

- `JC_LIBAVIF_SYS_PREBUILT_ONLY=1` cannot be combined with `JC_LIBAVIF_SYS_NO_PREBUILT=1`

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
