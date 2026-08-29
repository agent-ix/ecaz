---
task: 230
packet: 003-lifecycle-and-dml
agent: Codex
role: coder
model: gpt-5
date: 2026-08-29
seq: 5
---

# Task 230 packet 003 — row-tier I/O attribution checkpoint

Review code checkpoint `50701c204` against reviewer seq-04's must-land
heap/TOAST/tidx attribution requirement.

## Seq-05 row-tier I/O attribution scope

- Adds typed `task230_io_query_shape` values for id-only, cold-only, mixed, and
  select-all arms plus an explicit iteration count to both the suite schema and
  multinode CLI.
- Keeps one query shape per fresh fixture and rejects reuse, so cumulative I/O
  and cache state cannot bleed between shapes.
- Runs the selected end-to-end ANN projection for elapsed/materialized output,
  then runs the matching physical hot/row/cold projection independently on
  every owner.
- Takes each owner's pre/post `pg_statio_all_tables` snapshots and explicit
  `pg_stat_force_next_flush()` in the same backend session that performed the
  attributed relation reads.
- Emits heap, TOAST heap, and TOAST index read/hit deltas per relation, plus an
  aggregate shared-buffer hit ratio; counter resets and relation-identity drift
  fail closed.
- Adds a Task 230-only external, uncompressed TOAST fixture so cold/mixed/all
  shapes do not inherit the unrelated materialization-correctness variant
  matrix.

## Seq-05 validation

- `cargo fmt --all -- --check`: exit 0.
- Four focused ecaz-cli tests pass, including exact six-counter subtraction and
  fail-closed counter reset.
- Mandatory all-target PG18 clippy reproduces only the same five pre-existing
  findings; no finding is in the seq-05 changes.

## Seq-04 accepted scope

Reviewer seq-04 closed the multinode/suite selection and topology harness as
DONE. The accepted scope below remains for packet history.

## Seq-04 multinode and suite harness scope

- Adds explicit `--hot-cold-row-tier` and canonical
  `--hot-payload-attnums` options to `ecaz dev distann-multicluster`, including
  the frozen 1..=1536 dimension bound and fail-closed Task 229 sidecar
  exclusion.
- Extends the typed `ecaz bench suite` step schema and command expansion with
  the same options; no packet-local sweeper or raw-argument escape hatch is
  needed for the full-scale matrix.
- Reads all three cold-tier topology columns, rejects missing/incomplete pairs,
  and includes cold heap bytes in per-owner and aggregate generation storage.
- Attests hot/cold reloptions when reusing a fixture so a row-heap control
  cannot be silently reused as the candidate or vice versa.
- Closes reviewer seq-03's topology interpretation note: receipt digests are
  initial-content signals only before post-Ready DML; afterward graph
  current/tombstone state plus successful vec-id/schema-checked locator
  reconstruction is authoritative.

## Seq-04 validation

- `cargo fmt --all -- --check`: exit 0.
- Three focused ecaz-cli tests pass for canonical hot attnums, complete-pair
  topology validation, and typed suite expansion.
- Mandatory all-target PG18 clippy reproduces only the same five pre-existing
  findings; no finding is in the seq-04 changes.

## Seq-03 accepted scope

Reviewer seq-03 closed retained history and local destructive lifecycle as
DONE. The accepted scope below remains for packet history.

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

This closes the last harness prerequisite but is not packet-003 closure. Still
owed are the actual remote retry/intent and fault run,
publication/recovery and retained-generation reads, and restart/owner-failure
reads carried from packet 002.

## Review request

Please verify per-shape isolation, same-session pre/post/force-flush behavior,
all six `pg_statio_all_tables` deltas, aggregate hit-ratio math, and typed suite
expansion. Leave feedback under this packet's `feedback/` directory.
