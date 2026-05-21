# Review Request: SPIRE CustomScan Vector Query Expression Boundary

## Summary

Commit `97ebf0d3404ebe0dff223833d9b3045ba886ce86` removes the single-use `unsafe fn custom_scan_query_from_plan`.

The ORDER BY query expression decode now lives directly inside the vector `BeginCustomScan` unsafe boundary, next to tuple-payload state construction. That boundary already owns the live `CustomScanState`/provider-owned `CustomScan` contract and copies the decoded query into Rust-owned executor state before returning.

No safe raw-pointer helper was introduced.

## Unsafe Burndown

- `rg -n 'unsafe' src | wc -l`: `2521 -> 2519`
- Deleted:
  - `unsafe fn custom_scan_query_from_plan`
  - its internal unsafe expression-evaluation block

## Validation

See `artifacts/manifest.md`.

- `rustfmt --check src/am/ec_spire/custom_scan/dml.rs src/am/ec_spire/custom_scan/begin_exec.rs`
- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib custom_scan --no-default-features --features pg18,pg_test --no-run`

Known warnings only:

- stable-channel rustfmt import grouping warnings
- `src/am/mod.rs` unused SPIRE re-export warning
- Hadamard test-helper dead-code warnings
