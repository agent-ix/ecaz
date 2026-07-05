# Task 50 Packet 212: IVF Scan Descriptor View

## Summary

This slice consolidates IVF scan descriptor access behind a borrowed descriptor view.

Updated surface:

- `src/am/ec_ivf/scan.rs` now uses `IvfScanDescView<'_>` after validating the callback/debug `IndexScanDesc` pointer.
- Heap relation fallback, snapshot fallback, index relation access, and scan opaque access now go through safe methods on that view.
- The old `ivf_scan_opaque_option`, `ivf_scan_opaque_ref`, `unsafe fn resolve_scan_heap_relation`, and `unsafe fn resolve_scan_snapshot` contracts were removed.

The remaining unsafe in IVF scan is still around callback entry points, scan-owned raw allocations, palloc/pfree state, PostgreSQL relation/snapshot APIs, heap slot source decoding, and debug-only PG scan helpers.

## Counts

- `src`: `2559 -> 2557` unsafe references.
- `src/am/ec_ivf/scan.rs`: `50 -> 48` unsafe references.
- `src/am/ec_ivf/scan.rs`: `16 -> 13` unsafe function contracts.

## Validation

Artifacts are under `artifacts/` and indexed in `artifacts/manifest.md`.

- `rustfmt --check src/am/ec_ivf/scan.rs`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib ec_ivf --no-default-features --features pg18,pg_test --no-run`
- `git diff --check`

All validation passed. The cargo commands still report pre-existing unrelated warnings from `src/am/mod.rs` and Hadamard test helpers.
