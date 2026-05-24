# Task 50 Review Request: IVF Page Relation-View Callers

## Summary

This packet reviews commit
`39a5191a2dfd430fb82e53957d07dcbd4ffc3b10`, which continues the Task 50
unsafe burndown in the IVF/RaBitQ page layer.

The slice removes `7` direct unsafe blocks from `src/am/ec_ivf/page.rs`
(`29 -> 22`) by routing remaining page I/O call sites through the existing
`IvfPageRelation` contract.

## What Changed

- Replaced direct relation block-count reads with
  `IvfPageRelation::number_of_blocks`.
- Replaced direct `LockedBufferGuard::read_main` calls in posting summary,
  posting rewrite, tuple read, and tuple-tag scan paths with
  `IvfPageRelation::read_main`.
- Replaced direct `GenericXLogTxn::start` in posting rewrite with
  `IvfPageRelation::start_wal`.

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
| `src/` total direct unsafe blocks | 1928 | 1921 | -7 |
| `src/am/ec_ivf/page.rs` | 29 | 22 | -7 |
| `src/` unsafe ledger rows | 1928 | 1921 | -7 |

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1921` current `src/`
  unsafe rows.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`.
