# Third-party code

This repository does not commit upstream `libavif` or `libaom` source trees.

The top-level crate metadata declares `MIT OR Apache-2.0` for the original code in this repository. That declaration does not change the license terms of the downloaded upstream source code or the prebuilt native artifacts derived from it.

Pinned upstream versions live in `upstream/versions.toml`.

## `libavif`

- upstream project: `libavif`
- upstream repository: `https://github.com/AOMediaCodec/libavif`
- license family: BSD-2-Clause-style

## `libaom`

- upstream project: `libaom`
- upstream repository: `https://aomedia.googlesource.com/aom`
- license family: BSD-2-Clause-style plus the Alliance for Open Media patent grant

When this crate downloads sources locally, or when it consumes a published prebuilt archive, those upstream components keep their original licensing terms. The prebuilt archives produced by this repository include the corresponding upstream notice files under `licenses/`.
