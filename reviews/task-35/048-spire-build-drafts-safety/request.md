# Task 35 Packet 048: Spire Build Draft Safety

## Code Under Review

- Commit: `1d45f8a607eef5b430e35da9c0f9906cc9771993`
- Scope: `src/am/ec_spire/build/drafts.rs` and
  `scripts/unsafe_comment_baseline.txt`

## Summary

This slice documents the unsafe boundaries in SPIRE build draft and initial
publish helpers. It covers initial partitioned and recursive build publish
paths, relation-backed object-store opens, placement and manifest writes,
root/control initialization, backend-local publish timestamp reads, index tuple
Datum/null-array access, source-identity INCLUDE-column decoding, and UUID
payload byte access.

Key safety boundaries documented:

- backend-local timestamp reads for epoch publish windows
- live SPIRE index relation and local-store config used for build object stores
- relation-backed placement and manifest writes for initial publish
- root/control page initialization after manifest locators are written
- callback-owned values/isnull arrays and indexed key datum reads
- source-identity INCLUDE-column offset reads from validated `IndexInfo`
- UUID and bytea source-identity payload decoding without taking PostgreSQL
  ownership

## Baseline Accounting

- Global unsafe-comment baseline: `2545 -> 2526`
- `src/am/ec_spire/build/drafts.rs`: `19 -> 0`

## Validation

- `artifacts/unsafe-baseline-report-before.log`: before-count report showing
  `2545` global entries and `19 src/am/ec_spire/build/drafts.rs`.
- `artifacts/spire-build-drafts-baseline-before.log`: pre-slice baseline entry
  list ending with `entries: 19`.
- `artifacts/unsafe-audit-before-baseline-update.log`: unsafe-comment audit
  completed with exit code 0 before baseline regeneration.
- `artifacts/unsafe-baseline-update.log` and
  `artifacts/unsafe-baseline-update-after-fmt.log`: regenerated baseline logs,
  ending at `2526` entries.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh`
  completed with exit code 0 and no diagnostic output.
- `artifacts/unsafe-baseline-report-after.log`: after-count report showing
  `2526` global entries and no remaining build-drafts entry.
- `artifacts/spire-build-drafts-baseline-after.log`: after-count output showing
  `entries: 0`.
- `artifacts/unsafe-baseline-after-count.log`: after-count output showing
  `global: 2526` and `src/am/ec_spire/build/drafts.rs: 0`.
- `artifacts/git-diff-check.log`: `git diff --check` completed with exit code
  0 and no diagnostic output.
- `artifacts/cargo-fmt.log`: `cargo fmt --all` completed with the repository's
  existing stable-rustfmt warnings for unstable rustfmt options.
- `artifacts/cargo-check-pg18-bench.log`: cargo check completed successfully
  with known unrelated warnings.
- `artifacts/final-diff.patch`: final review diff.
