---
task: 230
packet: 003-lifecycle-and-dml
agent: Codex
role: coder
model: gpt-5
date: 2026-08-29
seq: 3
---

# Task 230 packet 003 — retained-history and destructive-lifecycle checkpoint

Review code checkpoint `7ff55c0a3` against reviewer seq-02's retained-history
semantics and remaining local lifecycle carry-ins.

## Seq-03 retained-history and destructive-lifecycle scope

- Documents that the topology orphan columns are raw physical-history counts:
  valid predecessor tuples retained for snapshot-pinned readers are included
  and are storage/churn attribution, not corruption by themselves.
- Extends the hot/cold DML callback to prove a healthy same-identity replacement
  reports exactly one retained hot tuple and one retained cold tuple.
- Proves `DROP INDEX` removes hot, cold, graph, and directory relations plus the
  generation catalog row.
- Proves `REINDEX` removes all four old generation relations and catalog state
  and mints a fresh logical index UUID.
- Proves an aborted `REINDEX` transaction restores all four relations, the cold
  catalog binding, and the original logical index UUID.

## Seq-03 validation

- `cargo fmt --all -- --check`: exit 0.
- Focused PG18 lifecycle group: two passed, zero failed.
- Mandatory all-target PG18 clippy: only the same five pre-existing findings;
  no finding in the seq-03 production documentation or lifecycle callback.

## Seq-02 accepted scope

Reviewer seq-02 closed the topology checkpoint as DONE. The accepted scope
below remains for packet history.

## Seq-02 topology scope

- Replaces the production topology diagnostic's hard-coded Graph V1 decode
  with descriptor-version dispatch, admitting hot/cold Graph V2 records.
- Opens and schema-validates the cold relation under the same inspection lock
  set as hot/graph/directory, reconstructs the frozen logical row from both
  compact tuples, and recomputes the unchanged logical row-tier digest.
- Preserves every existing topology column and appends explicit optional
  `cold_tier_row_count`, `cold_tier_orphan_row_count`, and `cold_tier_bytes`
  columns to both build-id and fingerprint endpoints. Legacy/sidecar
  generations report NULL for all three.
- Adds a published PG18 hot/cold topology callback that proves V2 admission,
  one hot plus one cold row, zero orphans in both tiers, byte accounting for
  both heaps, and logical digest equality with the frozen manifest.

## Seq-02 validation

- `cargo fmt --all -- --check`: exit 0.
- Focused PG18 topology callback: one passed, zero failed.
- Mandatory all-target PG18 clippy: only the same five pre-existing findings;
  no finding in `handoff.rs` or the new topology test.

## Seq-01 accepted scope

Reviewer seq-01 closed the DML/reclaim checkpoint as DONE. The accepted scope
below remains for packet history.

### DML and reclaim implementation

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

### Seq-01 validation

- `cargo fmt --all -- --check`: exit 0.
- `cargo pgrx test pg18 test_distann_hot_cold_ --no-default-features --features
  'pg18 pg_test'`: six focused PG18 callbacks pass, including both new tests
  and the four prior format/read-path hot/cold tests.
- The mandatory all-target PG18 clippy gate reports only the same five
  pre-existing failures recorded throughout packet 002; it reports no new
  production-DML or new-test lint.

## Packet status

This is a reviewable narrow checkpoint, not packet-003 closure. Local
DROP/REINDEX cleanup and rollback are now covered. Still owed are remote
retry/intent and fault coverage, publication/recovery and retained-generation
reads, and restart/owner-failure reads carried from packet 002.

## Review request

Please verify the raw orphan semantics and the complete hot/cold dependency
cleanup and rollback matrix. Leave feedback under this packet's `feedback/`
directory.
