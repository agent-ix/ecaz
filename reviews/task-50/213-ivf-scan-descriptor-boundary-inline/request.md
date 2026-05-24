# Task 50 Packet 213: IVF Scan Descriptor Boundary Inline

## Summary

This slice finishes the immediate IVF scan descriptor-view cleanup by removing the now-single-use raw descriptor helper.

Updated surface:

- `src/am/ec_ivf/scan.rs` now has `IvfScanDescView::from_raw` own the `IndexScanDesc` null check and borrowed descriptor construction directly.
- The old `unsafe fn ivf_scan_desc_ref` helper was removed.
- Existing scan descriptor access continues through `IvfScanDescView<'_>`.

The remaining unsafe in IVF scan is still around callback entry points, scan-owned raw allocations, PostgreSQL relation/snapshot APIs, heap slot source decoding, and debug-only PG scan helpers.

## Counts

- `src`: `2557 -> 2555` unsafe references.
- `src/am/ec_ivf/scan.rs`: `48 -> 46` unsafe references.
- `src/am/ec_ivf/scan.rs`: `13 -> 12` unsafe function contracts.

## Validation

Artifacts are under `artifacts/` and indexed in `artifacts/manifest.md`.

- `rustfmt --check src/am/ec_ivf/scan.rs`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib ec_ivf --no-default-features --features pg18,pg_test --no-run`
- `git diff --check`

All validation passed. The cargo commands still report pre-existing unrelated warnings from `src/am/mod.rs` and Hadamard test helpers.
