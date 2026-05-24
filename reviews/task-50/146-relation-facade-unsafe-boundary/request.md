# Task 50 Review Request: Relation Facade Unsafe Boundary

## Summary

This packet addresses the `storage/relation.rs` portion of the soundness audit
feedback in:

- `reviews/task-50/132-helper-soundness-audit/feedback/2026-05-20-02-reviewer.md`
- `reviews/task-50/132-helper-soundness-audit/feedback/2026-05-20-03-reviewer.md`

The shared relation metadata facade no longer exposes safe functions that take
raw `pg_sys::Relation` and dereference relation-owned fields. The raw-relation
helpers in `src/storage/relation.rs` are now `unsafe fn`, and direct callers
must acknowledge the live-relation precondition at their boundary.

This intentionally increases the direct unsafe count. The prior reduction hid a
PostgreSQL relation lifetime contract behind safe helper signatures; this packet
makes those contracts explicit again. It uses the audit's accepted Option A for
this slice. Option B guard/view inputs remain a possible future tightening, but
the current safe-fn-on-raw-relation facade is removed.

## Code

- code commit: `3f1b57a61fa4ef8ecfbb89e5d6b0caa8da792642`
- primary changed file: `src/storage/relation.rs`
- caller clusters updated: common planner/explain, DiskANN, HNSW, IVF, SPIRE,
  and HNSW recall test helpers.

## Validation

- `git diff --check HEAD^ HEAD`: passed
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the pre-existing `src/am/mod.rs` unused SPIRE DML import warning
- `make unsafe-block-count`: `1613` direct unsafe blocks/functions across `126` files
- `make unsafe-ledger ...`: generated `artifacts/unsafe-ledger-after.jsonl`
- `make unsafe-ledger-check ...`: `ledger covers 1613 current unsafe rows`

## Review Notes

This is a soundness correction, not a burndown-count reduction. It reverses
part of the cosmetic deletion identified by the reviewer by requiring explicit
unsafe context for relation metadata reads.

Remaining soundness-audit work includes scan/slot guard lifetimes and the
round-3 SPIRE/IVF helper clusters that still need the same strict treatment.
