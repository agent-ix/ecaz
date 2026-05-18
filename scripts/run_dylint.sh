#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLCHAIN="${ECAZ_DYLINT_TOOLCHAIN:-nightly-2026-04-16-aarch64-apple-darwin}"
CARGO_DYLINT="${CARGO_DYLINT:-$HOME/.cargo/bin/cargo-dylint}"

export RUSTUP_TOOLCHAIN="$TOOLCHAIN"
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
export DYLINT_DRIVER_PATH="${DYLINT_DRIVER_PATH:-$ROOT/crates/ecaz-lints/target/dylint-drivers}"
export DYLINT_RUSTFLAGS="${DYLINT_RUSTFLAGS:--D ecaz_panic_across_ffi}"

mkdir -p "$DYLINT_DRIVER_PATH"
cd "$ROOT"
exec "$CARGO_DYLINT" dylint --path crates/ecaz-lints "$@"
