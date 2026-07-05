# Task 35 Packet 049: Spire Custom Scan DML Safety

## Code Under Review

- Commit: `41f80400e3c086ec26d0b87bb3d0a318438e6554`
- Scope: `src/am/ec_spire/custom_scan/dml.rs` and
  `scripts/unsafe_comment_baseline.txt`

## Summary

This slice documents the unsafe boundaries in SPIRE custom-scan DML helper
logic. It covers provider-owned CustomScan plan-private offsets, custom
expression lists, executor state access, tuple payload column discovery,
attribute type I/O lookup, DML PK extraction, UPDATE expression evaluation,
Datum-to-JSON conversion, SPI-backed DELETE/UPDATE/PK SELECT primitive calls,
and the PK SELECT tuple-payload load transition.

Key safety boundaries documented:

- CustomScan plan-private offset 2 for LIMIT and custom_exprs offset 0 for
  ORDER BY / PK expressions
- live CustomScanState and CustomScan pointers supplied by the executor
- tuple descriptor and targetlist metadata reads for payload columns
- tuple attribute type metadata and input/receive function lookup
- NodeTag-dispatched Const/Param casts and expression evaluation
- integer Datum decoding by matching PostgreSQL type OID
- DML executor-state reads before SPI calls and payload-load state transition

## Baseline Accounting

- Global unsafe-comment baseline: `2526 -> 2509`
- `src/am/ec_spire/custom_scan/dml.rs`: `17 -> 0`

## Validation

- `artifacts/unsafe-baseline-report-before.log`: before-count report showing
  `2526` global entries and `17 src/am/ec_spire/custom_scan/dml.rs`.
- `artifacts/spire-custom-scan-dml-baseline-before.log`: pre-slice baseline
  entry list ending with `entries: 17`.
- `artifacts/unsafe-audit-before-baseline-update.log`: unsafe-comment audit
  completed with exit code 0 before baseline regeneration.
- `artifacts/unsafe-baseline-update.log` and
  `artifacts/unsafe-baseline-update-after-fmt.log`: regenerated baseline logs,
  ending at `2509` entries.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh`
  completed with exit code 0 and no diagnostic output.
- `artifacts/unsafe-baseline-report-after.log`: after-count report showing
  `2509` global entries and no remaining custom-scan DML entry.
- `artifacts/spire-custom-scan-dml-baseline-after.log`: after-count output
  showing `entries: 0`.
- `artifacts/unsafe-baseline-after-count.log`: after-count output showing
  `global: 2509` and `src/am/ec_spire/custom_scan/dml.rs: 0`.
- `artifacts/git-diff-check.log`: `git diff --check` completed with exit code
  0 and no diagnostic output.
- `artifacts/cargo-fmt.log`: `cargo fmt --all` completed with the repository's
  existing stable-rustfmt warnings for unstable rustfmt options.
- `artifacts/cargo-check-pg18-bench.log`: cargo check completed successfully
  with known unrelated warnings.
- `artifacts/final-diff.patch`: final review diff.
