---
task: 50
packet: 239
topic: diskann-cost-relation-guard-stats
role: coder
status: ready-for-review
created: 2026-05-21T06:51:40-07:00
head_sha: 82385aff1ffe6c9dfb6e8fb872b0907256d69508
---

# Review Request: DiskANN Cost Relation Guard Stats

## Summary

This packet reuses `DiskannInsertRelation` as the relation guard for DiskANN cost statistics.

Changes:

- Exposed `DiskannInsertRelation::main_fork_block_count` to sibling modules.
- Added `DiskannInsertRelation::reltuples`, keeping the raw relcache read behind the existing live-relation guard.
- Updated `ec_diskann` cost estimation and cost snapshot paths to read block count and `reltuples` through `DiskannInsertRelation`.
- Removed two direct `relation_reltuples(index_relation)` unsafe call sites from `src/am/ec_diskann/cost.rs`.

## Safety Notes

- `DiskannInsertRelation::from_raw` remains `unsafe fn`; callers still explicitly acknowledge that PostgreSQL supplied a live DISKANN relation.
- The new `reltuples` method relies on the same guard invariant as existing buffer/WAL helper methods on `DiskannInsertRelation`.
- Planner cost global reads remain `unsafe fn` per the restored round-1 audit convention.

## Unsafe Count

- `src/am/ec_diskann/cost.rs`: `9 -> 7`
- `src/am/ec_diskann/insert.rs`: `17 -> 18`
- Previous repo count: `2490`
- Current repo count: `2489`
- Delta: `-1`

The packet-local count log is:

- `artifacts/unsafe-counts.log`

## Validation

- `artifacts/rustfmt-check.log`: `rustfmt --check src/am/ec_diskann/cost.rs src/am/ec_diskann/insert.rs` passed with only known stable-rustfmt config warnings.
- `artifacts/git-diff-check.log`: `git diff --check HEAD^ HEAD` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known existing `src/am/mod.rs` unused SPIRE re-export warning.
- `artifacts/cargo-test-lib-ec-diskann-pg18-no-run.log`: `cargo test --lib ec_diskann --no-default-features --features pg18,pg_test --no-run` passed with the known existing Hadamard helper dead-code warnings.
