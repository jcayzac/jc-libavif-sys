//! Raw Rust FFI bindings for the public `libavif` C API.
//!
//! This crate exposes bindings generated from upstream `avif.h`, so its API
//! surface is intended to track `libavif` directly rather than wrap it in
//! higher-level Rust types.
//!
//! # Versioning
//!
//! The crate version is intended to track the pinned upstream `libavif`
//! version. For example, crate version `1.3.0` or `1.3.0-rc1` means this crate
//! is built against `libavif` `v1.3.0`.
//!
//! Native linking is handled by the crate build:
//!
//! - it prefers verified prebuilt native archives for supported targets
//! - if no matching prebuilt is available, it falls back to building the pinned
//!   upstream `libavif` and `libaom` sources locally
//!
//! The exported [`UPSTREAM_LIBAVIF_VERSION`] and [`UPSTREAM_LIBAOM_VERSION`]
//! constants identify the exact pinned upstream native versions that this crate
//! was built against.
//!
//! This crate is intended for low-level integration. If you want a more
//! idiomatic Rust API, build that in a separate wrapper crate on top of these
//! bindings.
//!
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

mod bindings;

pub use bindings::*;

/// The pinned upstream `libavif` version used by this crate build.
pub const UPSTREAM_LIBAVIF_VERSION: &str = env!("JC_LIBAVIF_SYS_UPSTREAM_LIBAVIF_VERSION");
/// The pinned upstream `libaom` version used by this crate build.
pub const UPSTREAM_LIBAOM_VERSION: &str = env!("JC_LIBAVIF_SYS_UPSTREAM_LIBAOM_VERSION");
