# Task 50 Review Request: Relation Tuple Descriptor Soundness Follow-Up

## Summary

This packet addresses reviewer soundness finding #3 from
`reviews/task-50/132-helper-soundness-audit/feedback/2026-05-20-01-reviewer.md`
and the related packet 126 cross-post.

The code checkpoint removes the safe `relation_tuple_desc` storage helper that
returned a borrowed raw `TupleDesc` without a lifetime boundary. The two SPIRE
custom scan callers now use `relation_tuple_desc_copy`, keeping tuple descriptor
metadata owned through `PgTupleDesc` while deriving the raw pointer only inside
the existing executor-local unsafe region.

The remaining IVF `heap_relation_tuple_desc` symbol is a separate local helper
inside a dirty IVF build file and is not the removed storage API.

## Code

- code commit: `d664d11b761a789f057b0609185704bdfe71c7af`
- changed files:
  - `src/storage/relation.rs`
  - `src/am/ec_spire/custom_scan/dml.rs`
  - `src/am/ec_spire/custom_scan/begin_exec.rs`

## Validation

- `git diff --check HEAD^ HEAD`: passed
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the pre-existing `src/am/mod.rs` unused SPIRE DML import warning
- `make unsafe-block-count`: `1550` direct unsafe blocks/functions across `124` files
- `make unsafe-ledger ...`: generated `artifacts/unsafe-ledger-after.jsonl`
- `make unsafe-ledger-check ...`: `ledger covers 1550 current unsafe rows`

## Review Notes

This is a soundness cleanup rather than a large burndown slice. It reduces the
packet 141 count from `1551` to `1550` and removes a safe API that exposed
borrowed PostgreSQL descriptor memory without encoding the relation lifetime.
