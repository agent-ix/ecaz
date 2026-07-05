# Review Request: SPIRE CustomScan Tuple Payload Column Discovery

## Summary

Commit `5f8e97d071289759db8728c0cb14a473a636cb9f` removes the single-use `unsafe fn custom_scan_tuple_payload_columns`.

Tuple-payload column discovery now lives inside `custom_scan_tuple_payload_state_from_plan`, the existing `BeginCustomScan` tuple-payload boundary. That boundary already owns the live `CustomScanState`/`CustomScan` contract and now reuses one copied tuple descriptor view for both:

- projected column discovery and validation;
- tuple-payload input metadata construction.

This removes a duplicate relation tuple descriptor copy and avoids adding any safe helper that accepts raw PostgreSQL pointers.

## Unsafe Burndown

- `rg -n 'unsafe' src | wc -l`: `2523 -> 2521`
- Deleted:
  - `unsafe fn custom_scan_tuple_payload_columns`
  - its internal unsafe relation/targetlist block

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
