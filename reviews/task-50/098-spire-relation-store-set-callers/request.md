# Task 50 Review Request: SPIRE Relation Store Set Callers

## Summary

This packet reviews commit
`f84ec0b6188bc5cd1ff383de6defe61ba8811837`, which makes
`SpireRelationObjectStoreSet::for_index_relation_and_placements` safe to call
and removes direct caller-side unsafe across SPIRE coordinator, production
scan-output, and vacuum paths.

The slice removes `7` direct unsafe blocks from `src/` (`1817 -> 1810`) and
drops `src/am/ec_spire/coordinator/diagnostics.rs` to zero direct unsafe blocks.

## What Changed

- Made the relation-backed object store set constructor safe to call.
- Kept the residual relation field read centralized in the constructor after
  the existing null check.
- Removed callers' direct unsafe wrappers in debug, diagnostics, active
  snapshot, production scan-output, and vacuum paths.

## Plan Coverage

This advances the comprehensive Task 50 plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`:

- P2 PostgreSQL handle views: store-set callers no longer own raw relation
  field access preconditions.
- P3 Buffer, Page, And WAL Transaction Contracts: relation-backed store opens
  are centralized behind the store set constructor and its relation guard.
- Wave 2 item 20: SPIRE remote-candidate coordinator views.

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
| `src/` total direct unsafe blocks | 1817 | 1810 | -7 |
| `src/am/ec_spire/coordinator/debug.rs` | 8 | 7 | -1 |
| `src/am/ec_spire/coordinator/diagnostics.rs` | 2 | 0 | -2 |
| `src/am/ec_spire/coordinator/remote_candidates/scan_output.rs` | 20 | 18 | -2 |
| `src/am/ec_spire/coordinator/snapshots.rs` | 10 | 9 | -1 |
| `src/am/ec_spire/vacuum/mod.rs` | 12 | 11 | -1 |
| `src/` unsafe ledger rows | 1817 | 1810 | -7 |

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1810` current `src/`
  unsafe rows.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan.
