# Task 50 Review Request: HNSW Scoped Opaque Debug Safe Helpers

## Summary

This slice removes the remaining `hnsw_scan_debug!` call sites from `src/tests/ec_hnsw_scan_gettuple.rs` and deletes the macro itself.

The code change adds scoped safe accessors for HNSW scan opaque inspection:

- `debug_with_scan_opaque`
- `debug_with_scan_opaque_mut`

Those helpers keep the raw descriptor-to-opaque dereference internal and lend the opaque reference only to a closure. This prevents safe debug helpers from accidentally carrying an opaque borrow across later AM callbacks. The change converts the remaining gettuple/frontier lifecycle debug helpers to safe public functions while keeping true graph/prefetch unsafe operations explicit at their local call sites.

## Unsafe Burndown

- Previous broad count from packet 267: `2247`
- Current broad count: `2237`
- Net: `-10`

After this packet, `src/tests/ec_hnsw_scan_gettuple.rs` has no remaining `hnsw_scan_debug` macro or call sites. The remaining `pub(crate) unsafe fn debug_*` items in `src/am/ec_hnsw/scan_debug.rs` are the older oracle helper group, not the gettuple/frontier debug API converted here.

## Validation

Artifacts are under `reviews/task-50/268-hnsw-scoped-opaque-debug-safe-helpers/artifacts/`.

- `git-diff-check.log`: passed
- `rustfmt-check.log`: passed; rustfmt emitted the existing stable-channel warnings for nightly-only import settings
- `hnsw-scan-debug-wrapper-grep.log`: no `hnsw_scan_debug` macro or call sites remain in the gettuple test include
- `unsafe-count.log`: `2237`
- `cargo-check-pg18-bench.log`: passed with the existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`
- `cargo-test-lib-pg18-pgtest-no-run.log`: passed with existing Hadamard test-only dead-code warnings

