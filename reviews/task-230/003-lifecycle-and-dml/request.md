---
task: 230
packet: 003-lifecycle-and-dml
agent: Codex
role: coder
model: gpt-5
date: 2026-08-29
seq: 1
---

# Task 230 packet 003 — hot/cold DML and reclaim checkpoint

Review code checkpoint `6d439e1e3` for the first packet-003 slice.

## Scope

- Physical inserts now map logical source attributes to the compact tier
  ordinals frozen in descriptor V4, append cold then hot, and publish a Graph
  V2 record only after both authoritative tuples exist in the same owner
  transaction.
- Forwarded owner payloads continue to use the full logical source schema on
  the wire. The owner decodes them into a source-heap slot, validates that
  schema against the retained descriptor, and only then partitions the row for
  storage. This keeps compact relation schemas out of the handoff contract.
- Candidate vector and stable-identity reads use the compact hot physical
  ordinals. Backlink read/modify/write, replacement, and tombstone paths
  dispatch through the retained graph-record version, preserving the V2 cold
  locator.
- Same-identity replacement appends a new cold/hot pair and graph version while
  retaining predecessor tuples. Delete changes only the current graph
  tombstone and leaves both locators and both tier heaps untouched.
- Adds PG18 coverage for exact hot/cold locator linkage, insert, replacement,
  graph-only tombstone, injected-failure rollback, retirement, reclaim
  rollback, idempotent reclaim, and dropping the cold tier with the generation.

## Validation

- `cargo fmt --all -- --check`: exit 0.
- `cargo pgrx test pg18 test_distann_hot_cold_ --no-default-features --features
  'pg18 pg_test'`: six focused PG18 callbacks pass, including both new tests
  and the four prior format/read-path hot/cold tests.
- The mandatory all-target PG18 clippy gate reports only the same five
  pre-existing failures recorded throughout packet 002; it reports no new
  production-DML or new-test lint.

## Packet status

This is a reviewable narrow checkpoint, not packet-003 closure. Still owed in
later checkpoints are retry/intent and remote-owner fault coverage, rebuild and
retained-predecessor recovery, restart and owner-failure reads carried from
packet 002, and the remaining drop/REINDEX lifecycle matrix.

## Review request

Please verify the logical-to-compact write mapping, cold-before-hot-before-graph
atomicity, V2 locator preservation across backlinks/replacement/delete, and
the retire/reclaim handling of the cold relation. Leave feedback under this
packet's `feedback/` directory.
