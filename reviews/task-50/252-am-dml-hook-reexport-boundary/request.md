# Task 50 Review Request: AM DML Hook Re-export Boundary

## Summary

Removed the last unsafe forwarding wrapper from `src/am/mod.rs`.

`am::register_dml_frontdoor_planner_hook` is now a direct crate-visible
re-export of the SPIRE implementation rather than a top-level wrapper that
only repeated the same extension-initialization contract and forwarded
immediately.

This leaves `src/am/mod.rs` with no remaining unsafe grep hits.

## Unsafe Burndown

- `src/am/mod.rs` unsafe grep count: `2 -> 0`
- repository `src` unsafe grep count: `2414 -> 2412`

See `artifacts/unsafe-counts.log`.

## Validation

- `rustfmt --edition 2021 --check src/am/mod.rs`
  - Passed; stable rustfmt emitted the existing unstable-option warnings.
- `git diff --check`
  - Passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - Passed; emitted the existing unused SPIRE re-export warning in
    `src/am/mod.rs`.
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
  - Passed; emitted the existing Hadamard test-helper dead-code warnings.

## Review Focus

Please verify the re-export preserves the `am::register_dml_frontdoor_planner_hook`
name used during extension initialization and does not change the underlying
SPIRE hook ownership contract.
