# Task 35 Packet 045: Spire Insert Safety

## Code Under Review

- Commit: `3a674dcdb49eeb3d1e9fc6bd404d6eb15e3a3d97`
- Scope: `src/am/ec_spire/insert.rs` and `scripts/unsafe_comment_baseline.txt`

## Summary

This slice documents the unsafe boundaries in the SPIRE `aminsert` path. It
covers the PostgreSQL AM callback guard, publish-lock/root-control setup,
tuple-layout and tuple construction from callback inputs, active-epoch manifest
loads, relation-backed store opens, publish timestamps, placement writes,
replacement publish, and empty-index bootstrap publish.

Key safety boundaries documented:

- PostgreSQL `aminsert` callback input lifetime for relations, Datum/null
  arrays, heap TID, and `IndexInfo`
- live SPIRE index relation assumptions for publish locking, root/control page
  reads, relation options, local-store config, manifest loads, and object-store
  opens
- tuple layout and tuple construction from validated callback metadata
- backend-local publish timestamp reads
- publish-lock requirements for placement, manifest, and root/control writes
- bootstrap relcache OID/tablespace reads used to seed embedded store config

## Baseline Accounting

- Global unsafe-comment baseline: `2605 -> 2584`
- `src/am/ec_spire/insert.rs`: `21 -> 0`

## Validation

- `artifacts/unsafe-baseline-report-before.log`: before-count report showing
  `2605` global entries and `21 src/am/ec_spire/insert.rs`.
- `artifacts/spire-insert-baseline-before.log`: pre-slice insert baseline
  entry list ending with `entries: 21`.
- `artifacts/unsafe-audit-before-baseline-update.log`: unsafe-comment audit
  completed with exit code 0 before baseline regeneration.
- `artifacts/unsafe-baseline-update.log` and
  `artifacts/unsafe-baseline-update-after-fmt.log`: regenerated baseline logs,
  ending at `2584` entries.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh`
  completed with exit code 0 and no diagnostic output.
- `artifacts/unsafe-baseline-report-after.log`: after-count report showing
  `2584` global entries and no remaining insert entry.
- `artifacts/spire-insert-baseline-after.log`: after-count output showing
  `entries: 0`.
- `artifacts/unsafe-baseline-after-count.log`: after-count output showing
  `global: 2584` and `src/am/ec_spire/insert.rs: 0`.
- `artifacts/git-diff-check.log`: `git diff --check` completed with exit code
  0 and no diagnostic output.
- `artifacts/cargo-fmt.log`: `cargo fmt --all` completed with the repository's
  existing stable-rustfmt warnings for unstable rustfmt options.
- `artifacts/cargo-check-pg18-bench.log`: cargo check completed successfully
  with known unrelated warnings.
- `artifacts/final-diff.patch`: final review diff.
