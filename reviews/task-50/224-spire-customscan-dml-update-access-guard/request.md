# Review Request: SPIRE CustomScan DML Update Access Guard

## Summary

Commit `4836bba01d24300aa2dfc8b5bb1bf8cef2dfcc4a` threads the existing `CustomScanAccessState<'_>` view into `custom_scan_execute_dml_update`.

Before this slice, `custom_scan_execute_dml_update` accepted a raw `pg_sys::ScanState` and was marked `unsafe`; the only caller immediately passed `access_state.as_ptr()` through an unsafe call. The helper now accepts `CustomScanAccessState<'_>` directly, keeping the live executor-state contract in the typed access-state view constructed at the PostgreSQL callback boundary.

The remaining unsafe block in `custom_scan_execute_dml_update` is still real: it casts the guarded scan state to `CustomScanState` and evaluates provider-owned UPDATE expressions through PostgreSQL executor APIs.

## Unsafe Burndown

- `rg -n 'unsafe' src | wc -l`: `2531 -> 2529`
- Removed:
  - `unsafe fn custom_scan_execute_dml_update`
  - the caller-side `unsafe { custom_scan_execute_dml_update(...) }`

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
