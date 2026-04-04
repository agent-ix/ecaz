---
id: NFR-005
title: Build and CI
type: non-functional-requirement
status: APPROVED
traces:
  - StR-002
---
# NFR-005: Build and CI

## Requirement

### Toolchain

- Rust stable (MSRV: 1.75+)
- pgrx 0.12+
- Clippy: all warnings are errors (`-D warnings`)
- rustfmt: enforced in CI

### CI Pipeline

1. `cargo fmt --check` — formatting
2. `cargo clippy --all-targets --all-features -- -D warnings` — lint
3. `cargo test` — unit tests (no Postgres required)
4. `cargo pgrx test pg17` — integration tests (Postgres required)
5. `cargo deny check licenses` — license audit

### Build Targets

The extension SHALL build for:
- `x86_64-unknown-linux-gnu` (primary)
- `aarch64-unknown-linux-gnu` (ARM64 servers)

AVX2 SIMD is enabled by default (`-C target-cpu=native`) for development but SHALL NOT be hard-required — the extension SHALL compile (with degraded performance) without AVX2.

## Measurement

CI pipeline runs on every push and PR. All steps must pass for merge.
