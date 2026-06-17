---
id: NFR-005
title: Build and CI
type: NFR
status: APPROVED
traces:
  - StR-002
---
# NFR-005: Build and CI

## Statement

Ecaz SHALL build and pass its continuous-integration gates on every push and pull request across the supported toolchain and target matrix below.

### Toolchain

- Rust stable
- pgrx 0.17+
- Clippy: all warnings are errors (`-D warnings`)
- rustfmt: enforced in CI

### CI Pipeline

1. `cargo fmt --check` — formatting
2. `cargo clippy --all-targets --all-features -- -D warnings` — lint
3. `cargo test` — unit tests (no Postgres required)
4. `cargo pgrx test pg18` — primary integration lane (Postgres required)
5. `cargo pgrx test pg17` — compatibility integration lane when PG17 coverage is requested
6. `cargo deny check licenses` — license audit

### Build Targets

The extension SHALL build for:
- PostgreSQL 18 as the primary target
- PostgreSQL 17 as the compatibility fallback
- `x86_64-unknown-linux-gnu` (primary)
- `aarch64-unknown-linux-gnu` (ARM64 servers)

AVX2 SIMD is enabled by default (`-C target-cpu=native`) for development but SHALL NOT be hard-required — the extension SHALL compile (with degraded performance) without AVX2.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| CI pipeline pass rate on push/PR | 100% of required steps | 100% | CI Gate |
| PG18 primary integration lane (`cargo pgrx test pg18`) | Pass | Pass | Integration Test |
| Build without AVX2 | Compiles (degraded perf) | Compiles | Build Test |


CI pipeline runs on every push and PR. All steps must pass for merge.

## Verification

The CI pipeline runs `cargo fmt --check`, clippy with `-D warnings`, `cargo test`, `cargo pgrx test pg18`, and `cargo deny check licenses` on every push and PR; all required steps must pass for merge, and a no-AVX2 build is exercised to confirm the extension still compiles.

