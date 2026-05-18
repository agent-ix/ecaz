#!/usr/bin/env bash
set -euo pipefail

export RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-nightly-2026-04-16-aarch64-apple-darwin}"
export PATH="$HOME/.cargo/bin:$PATH"

exec "${DYLINT_LINK:-dylint-link}" "$@"
