#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <target-triple> <output-dir>" >&2
  exit 1
fi

target="$1"
output_dir="$2"

mkdir -p "$output_dir"

export JC_LIBAVIF_SYS_NO_PREBUILT=1

cargo build --release --target "$target"

search_roots=()
if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
  search_roots+=("${CARGO_TARGET_DIR}/release/build")
  search_roots+=("${CARGO_TARGET_DIR}/${target}/release/build")
else
  search_roots+=("target/${target}/release/build")
  search_roots+=("target/release/build")
fi

install_dir="$(find "${search_roots[@]}" -path '*/out/libavif-install' -type d 2>/dev/null | head -n 1)"

if [[ -z "$install_dir" ]]; then
  echo "could not find libavif-install output directory" >&2
  exit 1
fi

archive="$output_dir/jc-libavif-sys-native-${target}.tar.gz"

tar -czf "$archive" -C "$install_dir" .

if command -v sha256sum >/dev/null 2>&1; then
  sha256sum "$archive" > "${archive}.sha256"
else
  shasum -a 256 "$archive" > "${archive}.sha256"
fi
