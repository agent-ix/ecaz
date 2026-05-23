---
task: 50
packet: 127-index-heap-relation-boundary
role: coder
agent: codex
model: GPT-5
date: 2026-05-20
head_sha: 98142cb52ae9e3ad6b868e51b3fdab4227059b19
code_commit: 98142cb52ae9e3ad6b868e51b3fdab4227059b19
status: ready-for-review
---

# Review Request: Index Heap Relation Boundary

## Summary

This slice centralizes index-to-heap relation OID resolution behind storage
relation helpers:

- `relation_oid`
- `index_heap_relation_oid`
- `index_heap_relation_oid_from_index_oid`

It removes repeated direct `IndexGetRelation` unsafe blocks from SPIRE, HNSW,
and DiskANN call sites. Mixed scan-descriptor sites still retain only the raw
scan pointer read locally, while heap OID resolution now goes through the
central storage helper.

The remaining direct `IndexGetRelation` call sites are:

- `src/storage/relation.rs`: the centralized residual FFI boundary.
- `src/am/ec_ivf/scan.rs`: left untouched because that file already has
  unrelated local edits in the worktree.

## Unsafe Burndown

- Previous packet count: `1607` unsafe blocks across `123` files.
- This packet count: `1596` unsafe blocks across `123` files.
- Net change: `-11` direct unsafe blocks.

## Validation

Artifacts are under `reviews/task-50/127-index-heap-relation-boundary/artifacts/`.

- `git-diff-check.log`: clean.
- `cargo-check-pg18-bench.log`: pass for `cargo check --all-targets --no-default-features --features pg18,bench`.
- `unsafe-ledger-check.log`: `ledger covers 1596 current unsafe rows`.
- `count-summary.md`: `unsafe_blocks 1596`, `files 123`.

Known residual: cargo still reports the pre-existing `src/am/mod.rs` SPIRE DML
unused-import warning; this slice does not touch those imports.
