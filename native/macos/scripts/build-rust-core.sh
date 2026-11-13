#!/bin/bash
# Xcode build phase: builds crates/justslide-ffi as a static library for each
# architecture being built and lipo-merges it into BUILT_PRODUCTS_DIR.
set -euo pipefail
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"
REPO="$(cd "$SRCROOT/../.." && pwd)"
if ! command -v cargo >/dev/null; then
  echo "error: Rust toolchain not found. Install via https://rustup.rs (rust-toolchain.toml pins the version), then rebuild." >&2
  exit 1
fi
PROFILE=release; DIR=release; FEATURES=()
if [ "${CONFIGURATION:-Debug}" = "Debug" ]; then PROFILE=dev; DIR=debug; FEATURES=(--features design-fixture); fi
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-14.0}"
LIBS=()
for ARCH in ${ARCHS:-$(uname -m)}; do
  case "$ARCH" in x86_64) T=x86_64-apple-darwin ;; arm64) T=aarch64-apple-darwin ;; *) echo "error: unsupported arch $ARCH" >&2; exit 1 ;; esac
  rustup target add "$T" >/dev/null 2>&1 || true
  cargo build --manifest-path "$REPO/Cargo.toml" -p justslide-ffi --profile "$PROFILE" --target "$T" "${FEATURES[@]}"
  LIBS+=("$REPO/target/$T/$DIR/libjustslide_ffi.a")
done
mkdir -p "$BUILT_PRODUCTS_DIR"
lipo -create "${LIBS[@]}" -output "$BUILT_PRODUCTS_DIR/libjustslide_ffi.a"
echo "justslide-ffi -> $BUILT_PRODUCTS_DIR/libjustslide_ffi.a (${ARCHS:-host})"
