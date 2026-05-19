# Task 35 Packet 044: Spire Cost Safety

## Code Under Review

- Commit: `6ab741a7c424ba3167a63aa92c05713ace9120a2`
- Scope: `src/am/ec_spire/cost/mod.rs` and `scripts/unsafe_comment_baseline.txt`

## Summary

This slice documents the unsafe boundaries in the SPIRE cost-model callbacks
and snapshot helpers. It covers the PostgreSQL planner cost callback, live
relation block-count and relcache reads, planner cost constant reads, relation
option and metadata snapshots, PG18 tree-height callback, and PG18
strategy/compare-type translation callbacks.

Key safety boundaries documented:

- PostgreSQL `amcostestimate` callback with live `IndexPath` and output
  pointers
- live SPIRE index relation assumptions for block counts and relcache
  `reltuples`
- planner cost GUC reads in the current backend
- live relation option, active snapshot diagnostic, and hierarchy snapshot
  reads
- tree-height callback value derived from SPIRE hierarchy metadata
- PG18 scalar AM callbacks that execute inside `pgrx_extern_c_guard`

## Baseline Accounting

- Global unsafe-comment baseline: `2627 -> 2605`
- `src/am/ec_spire/cost/mod.rs`: `22 -> 0`

## Validation

- `artifacts/unsafe-baseline-report-before.log`: before-count report showing
  `2627` global entries and `22 src/am/ec_spire/cost/mod.rs`.
- `artifacts/spire-cost-baseline-before.log`: pre-slice cost baseline entry
  list.
- `artifacts/unsafe-baseline-update.log` and
  `artifacts/unsafe-baseline-update-after-fmt.log`: regenerated baseline logs,
  ending at `2605` entries.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh`
  completed with exit code 0 and no diagnostic output.
- `artifacts/unsafe-baseline-report-after.log`: after-count report showing
  `2605` global entries and no remaining cost entry.
- `artifacts/spire-cost-baseline-after.log`: after-count output showing
  `entries: 0`.
- `artifacts/unsafe-baseline-after-count.log`: after-count output showing
  `global: 2605` and `src/am/ec_spire/cost/mod.rs: 0`.
- `artifacts/git-diff-check.log`: `git diff --check` completed with exit code
  0 and no diagnostic output.
- `artifacts/cargo-fmt.log`: `cargo fmt --all`.
- `artifacts/cargo-check-pg18-bench.log`: cargo check completed successfully
  with known unrelated warnings.
- `artifacts/final-diff.patch`: final review diff.
