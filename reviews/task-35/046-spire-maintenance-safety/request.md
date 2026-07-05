# Task 35 Packet 046: Spire Maintenance Safety

## Code Under Review

- Commit: `39a4f4389051c536f00133433677702eabc11693`
- Scope: `src/am/ec_spire/coordinator/maintenance.rs` and
  `scripts/unsafe_comment_baseline.txt`

## Summary

This slice documents the unsafe boundaries in SPIRE scheduled-maintenance
planning and publish execution. It covers publish-lock acquisition, root/control
page reads, active epoch manifest loads, relation-backed object-store opens,
split replacement heap-source reconstruction, publish timestamp reads, and
scheduled replacement epoch publishing.

Key safety boundaries documented:

- live SPIRE index relation assumptions for maintenance planning and execution
- publish-lock serialization for maintenance plan/run and replacement publish
- active root/control metadata as the source of epoch manifest tuple locators
- relation-backed object-store opens used for candidate inspection and publish
- heap relation, active snapshot, tuple slot, and indexed attribute lifetimes
  while constructing split replacement input
- backend-local publish timestamp reads
- scheduled replacement input and publish tied to the same selected epoch plan

## Baseline Accounting

- Global unsafe-comment baseline: `2584 -> 2564`
- `src/am/ec_spire/coordinator/maintenance.rs`: `20 -> 0`

## Validation

- `artifacts/unsafe-baseline-report-before.log`: before-count report showing
  `2584` global entries and `20 src/am/ec_spire/coordinator/maintenance.rs`.
- `artifacts/spire-maintenance-baseline-before.log`: pre-slice maintenance
  baseline entry list ending with `entries: 20`.
- `artifacts/unsafe-audit-before-baseline-update.log`: unsafe-comment audit
  completed with exit code 0 before baseline regeneration.
- `artifacts/unsafe-baseline-update.log` and
  `artifacts/unsafe-baseline-update-after-fmt.log`: regenerated baseline logs,
  ending at `2564` entries.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh`
  completed with exit code 0 and no diagnostic output.
- `artifacts/unsafe-baseline-report-after.log`: after-count report showing
  `2564` global entries and no remaining maintenance entry.
- `artifacts/spire-maintenance-baseline-after.log`: after-count output showing
  `entries: 0`.
- `artifacts/unsafe-baseline-after-count.log`: after-count output showing
  `global: 2564` and `src/am/ec_spire/coordinator/maintenance.rs: 0`.
- `artifacts/git-diff-check.log`: `git diff --check` completed with exit code
  0 and no diagnostic output.
- `artifacts/cargo-fmt.log`: `cargo fmt --all` completed with the repository's
  existing stable-rustfmt warnings for unstable rustfmt options.
- `artifacts/cargo-check-pg18-bench.log`: cargo check completed successfully
  with known unrelated warnings.
- `artifacts/final-diff.patch`: final review diff.
