# Task 50 Review Request: HNSW Relation Options Callers

## Summary

This packet reviews commit
`4599f93c4a871359f58a48feb142563b3099c483`, which makes
`ec_hnsw::options::relation_options` safe to call and removes caller-side
unsafe wrappers across HNSW build, graph, insert, scan, shared diagnostics/cost,
vacuum, and common planner cost code.

The slice removes `9` direct unsafe blocks from `src/` (`1778 -> 1769`).

## What Changed

- Made HNSW `relation_options` safe to call.
- Added a null relation guard before reading the relation descriptor.
- Kept raw `rd_options`, reloption struct casts, and string-offset reads
  centralized in `src/am/ec_hnsw/options.rs`.
- Removed simple caller-side unsafe wrappers across HNSW production and
  diagnostic surfaces.

## Plan Coverage

This advances the comprehensive Task 50 plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`:

- P2 PostgreSQL handle views: HNSW reloption reads no longer require callers to
  encode relation-pointer preconditions.
- P7 Reloptions And C String Contracts: HNSW reloptions now have a safe API
  boundary and a named residual owner.
- Wave 3 item 33: HNSW/DiskANN options/reloptions cleanup.

## Evidence

- Code diff: `artifacts/code-diff.patch`
- Validation: `artifacts/cargo-check-pg18-bench.log`
- Whitespace check: `artifacts/git-diff-check.log`
- Unsafe count: `artifacts/src-unsafe-block-count-after.log`
- Count summary: `artifacts/count-summary.md`
- Ledger: `artifacts/unsafe-ledger-after.jsonl`
- Ledger generation/check logs:
  `artifacts/unsafe-ledger-generate.log`,
  `artifacts/unsafe-ledger-check.log`

## Result

Direct unsafe movement:

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/` total direct unsafe blocks | 1778 | 1769 | -9 |
| `src/am/common/cost.rs` | 14 | 13 | -1 |
| `src/am/ec_hnsw/build.rs` | 30 | 29 | -1 |
| `src/am/ec_hnsw/graph.rs` | 56 | 55 | -1 |
| `src/am/ec_hnsw/insert.rs` | 73 | 72 | -1 |
| `src/am/ec_hnsw/scan.rs` | 146 | 145 | -1 |
| `src/am/ec_hnsw/shared.rs` | 42 | 39 | -3 |
| `src/am/ec_hnsw/vacuum.rs` | 65 | 64 | -1 |
| `src/` unsafe ledger rows | 1778 | 1769 | -9 |

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1769` current `src/`
  unsafe rows.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan.
