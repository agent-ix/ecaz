# Task 35 Review Request: IVF Scan Callback State Safety

## Summary

Code commit under review: `b914629f9ab61790d5ff09b48fb6d55ecfd2c882`

This slice starts the `src/am/ec_ivf/scan.rs` unsafe burndown by documenting
the scan opaque accessors, AM callback guard boundaries, and PostgreSQL scan
output slot writes.

The covered areas are:

- `EcIvfScanOpaque` query, selected-list, and posting-candidate accessors
- `ec_ivf_ambeginscan`
- `ec_ivf_amrescan`
- `ec_ivf_amgettuple`
- `ec_ivf_amendscan`
- `set_scan_heap_tid`
- `set_scan_orderby_score`
- `clear_scan_orderby_output`

The added `SAFETY:` comments cover allocation/count invariants for scan-local
buffers, callback invocation and `pgrx_extern_c_guard` unwind containment, and
single-slot ORDER BY output writes through PostgreSQL-owned scan descriptor
storage.

## Baseline Accounting

- Global unsafe baseline: `2821 -> 2810`
- `src/am/ec_ivf/scan.rs`: `101 -> 90`

## Validation

- `bash scripts/check_unsafe_comments.sh` passed with an empty log:
  `artifacts/unsafe-audit-after.log`
- `make unsafe-baseline-report` reports `2810` entries and IVF scan at `90`:
  `artifacts/unsafe-baseline-report-after.log`
- `cargo fmt --all` ran; known unrelated format churn was restored before
  final validation: `artifacts/cargo-fmt.log`
- `git diff --check` passed with an empty log:
  `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  passed with the existing unrelated warnings in `src/am/common/parallel.rs`
  and `src/am/mod.rs`: `artifacts/cargo-check-pg18-bench.log`

## Artifacts

See `artifacts/manifest.md` for command lines, timestamps, and packet-local
evidence files.
