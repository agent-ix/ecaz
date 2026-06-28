---
id: NFR-005
title: Build and CI
type: NFR
quality_attribute: maintainability
status: APPROVED
traces:
  - StR-002
---
# NFR-005: Build and CI

## Statement

The extension SHALL build on Rust stable with pgrx 0.17+, pass the full CI
pipeline (formatting, lint, unit tests, pgrx integration lanes, license audit)
on every push and PR, and build for the declared PostgreSQL and architecture
targets.

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
|---|---|---|---|
| CI pipeline steps passing per push/PR | all steps pass | any failing step blocks merge | CI pipeline run on every push and PR |
| Formatting drift | none | `cargo fmt --check` passes | `cargo fmt --check` CI step |
| Clippy warnings | zero | `-D warnings` (all warnings are errors) | `cargo clippy --all-targets --all-features -- -D warnings` CI step |
| Unit test failures | zero | `cargo test` passes | `cargo test` CI step |
| PG18 integration test failures | zero | `cargo pgrx test pg18` passes | `cargo pgrx test pg18` CI step |
| License audit findings | zero | `cargo deny check licenses` passes | `cargo deny check licenses` CI step |
| Build target coverage | PG18 primary + PG17 fallback, x86_64 and aarch64 linux; compiles without AVX2 | build succeeds (degraded performance without AVX2 acceptable) | CI build matrix |

CI pipeline runs on every push and PR. All steps must pass for merge.

## Verification

Compliance is checked by the CI pipeline itself: every push and PR runs
`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
`cargo test`, `cargo pgrx test pg18` (with `cargo pgrx test pg17` when PG17
coverage is requested), and `cargo deny check licenses`. Merge is blocked
unless all steps pass. Build-target compliance is verified by building for
PostgreSQL 18/17 on `x86_64-unknown-linux-gnu` and
`aarch64-unknown-linux-gnu`, including a build without AVX2.
