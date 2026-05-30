# Task 67 Review Request: MSRV Clippy allowance for RaBitQ AVX-512

## Summary

This packet fixes the PR #8 x86_64 CI failure that appeared after rebasing the Task 33 branch onto current `main`.

The first failing job was `Build Matrix / x86_64-unknown-linux-gnu`, step `cargo clippy`. Clippy reported `clippy::incompatible_msrv` errors for AVX-512 intrinsics in `src/quant/rabitq.rs`; those intrinsics are newer than the crate MSRV but are compiled behind target-feature-gated SIMD paths.

The follow-up rerun also exposed a CI `cargo fmt --all -- --check` failure in `crates/ecaz-cli/src/commands/corpus/load.rs`. This packet includes the exact wrapping change shown by the CI formatter diff.

## Change

- Adds `#![allow(clippy::incompatible_msrv)]` at the RaBitQ module level.
- Applies the CI-required rustfmt wrapping in the corpus chunked-manifest validation error messages.
- Leaves runtime code unchanged.
- Keeps the allowance scoped to the module that owns the target-gated RaBitQ SIMD kernels.

## Validation

- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` passed locally on macOS/aarch64; see `artifacts/cargo-clippy-local.log`.
- `git diff --check` passed; see `artifacts/git-diff-check.log`.
- Local `cargo fmt --all -- --check` used `rustfmt 1.9.0` and disagreed with CI's Rust 1.95 formatter on the same wrapping; see `artifacts/cargo-fmt-check.log`. The checked-in formatting follows the CI diff from run `26691249527`, job `78667910979`.

The exact failing surfaces are Linux x86_64 CI and CI's Rust 1.95 formatter, so the authoritative validation for this packet is the rerun triggered by pushing this branch.
