# Task 35 Packet 050: Spire Custom Scan Begin/Exec Safety

## Code Under Review

- Commit: `ba2ee9b3c8d9f613f066272479bf1f8e2674f43e`
- Scope: `src/am/ec_spire/custom_scan/begin_exec.rs` and
  `scripts/unsafe_comment_baseline.txt`

## Summary

This slice documents the unsafe boundaries in SPIRE custom-scan executor
lifecycle wiring. It covers custom scan state allocation, default state
initialization, Begin/Exec/End/ReScan callbacks, tuple-payload descriptor setup,
test-only memory-context accounting, ExecScan access/recheck wiring, vector
output access, and DML PK SELECT/UPDATE/DELETE access paths.

Key safety boundaries documented:

- executor-lifetime `palloc0` allocation and Rust state initialization
- zeroed PostgreSQL `CustomScanState` C struct before executor setup fills it
- live CustomScanState/CustomScan plan ownership in BeginCustomScan
- scan relation tuple descriptor lifetime during tuple-payload setup
- ExecScan callback ownership of tuple slot handling
- EndCustomScan one-time Drop + `pfree` lifecycle for provider state
- ReScan cast back to provider state and reset of loaded-output flags
- DML access paths performing one PK SELECT/UPDATE/DELETE emission per state

## Baseline Accounting

- Global unsafe-comment baseline: `2509 -> 2496`
- `src/am/ec_spire/custom_scan/begin_exec.rs`: `13 -> 0`

## Validation

- `artifacts/unsafe-baseline-report-before.log`: before-count report showing
  `2509` global entries and `13 src/am/ec_spire/custom_scan/begin_exec.rs`.
- `artifacts/spire-custom-scan-begin-exec-baseline-before.log`: pre-slice
  baseline entry list ending with `entries: 13`.
- `artifacts/unsafe-audit-before-baseline-update.log`: unsafe-comment audit
  completed with exit code 0 before baseline regeneration.
- `artifacts/unsafe-baseline-update.log` and
  `artifacts/unsafe-baseline-update-after-fmt.log`: regenerated baseline logs,
  ending at `2496` entries.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh`
  completed with exit code 0 and no diagnostic output.
- `artifacts/unsafe-baseline-report-after.log`: after-count report showing
  `2496` global entries and no remaining custom-scan begin/exec entry.
- `artifacts/spire-custom-scan-begin-exec-baseline-after.log`: after-count
  output showing `entries: 0`.
- `artifacts/unsafe-baseline-after-count.log`: after-count output showing
  `global: 2496` and `src/am/ec_spire/custom_scan/begin_exec.rs: 0`.
- `artifacts/git-diff-check.log`: `git diff --check` completed with exit code
  0 and no diagnostic output.
- `artifacts/cargo-fmt.log`: `cargo fmt --all` completed with the repository's
  existing stable-rustfmt warnings for unstable rustfmt options.
- `artifacts/cargo-check-pg18-bench.log`: cargo check completed successfully
  with known unrelated warnings.
- `artifacts/final-diff.patch`: final review diff.
