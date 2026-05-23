# Task 50 Review Request: DiskANN Relation Options Callers

## Summary

This packet reviews commit
`fe633d4760de5642c004ece43846d9fea63c24dd`, which makes
`ec_diskann::options::relation_options` safe to call and removes caller-side
unsafe wrappers across DiskANN build, cost, insert, and test scan-state setup
surfaces.

The slice removes `5` direct unsafe blocks from `src/` (`1769 -> 1764`).

## What Changed

- Made DiskANN `relation_options` safe to call.
- Added a null relation guard before reading the relation descriptor.
- Kept raw `rd_options`, reloption struct casts, and string-offset reads
  centralized in `src/am/ec_diskann/options.rs`.
- Removed simple caller-side unsafe wrappers across DiskANN production,
  diagnostic, and test setup surfaces.

## Plan Coverage

This advances the comprehensive Task 50 plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`:

- P2 PostgreSQL handle views: DiskANN reloption reads no longer require callers
  to encode relation-pointer preconditions.
- P7 Reloptions And C String Contracts: DiskANN reloptions now have a safe API
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
| `src/` total direct unsafe blocks | 1769 | 1764 | -5 |
| `src/am/ec_diskann/ambuild.rs` | 38 | 37 | -1 |
| `src/am/ec_diskann/cost.rs` | 5 | 3 | -2 |
| `src/am/ec_diskann/insert.rs` | 40 | 39 | -1 |
| `src/am/ec_diskann/routine.rs` | 56 | 55 | -1 |
| `src/` unsafe ledger rows | 1769 | 1764 | -5 |

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check fe633d47^ fe633d47`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1764` current `src/`
  unsafe rows.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan.
