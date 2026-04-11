# Review Request: A5 Insert Drift Accounting

## Context

Branch:
- `main`

Task / roadmap inputs:
- `plan/tasks/06-graph-insert.md`
- `review/199-aminsert-graph-aware-insertion-roadmap/request.md`
- `plan/status.md`
- `spec/functional/FR-016-hnsw-insert.md`
- `spec/adr/ADR-026-live-insert-backlink-lock-ordering.md`

This is the next narrow A5 checkpoint after overflow backlink pruning. It lands
the drift-accounting portion of A5 without trying to solve the final
concurrency-hardening slice in the same jump.

Checkpoint scope:

1. persist `inserted_since_rebuild` in metadata
2. reset that counter on bulk build / REINDEX output
3. increment it on successful live inserts only
4. expose the counter and derived drift fraction through the SQL/admin snapshot
5. update task/spec/status docs to reflect that drift observability is now live

## Scope

- `src/am/page.rs`
- `src/am/build.rs`
- `src/am/insert.rs`
- `src/am/shared.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `src/am/scan.rs`
- `tests/proptest_page.rs`
- `plan/tasks/06-graph-insert.md`
- `plan/status.md`
- `spec/functional/FR-016-hnsw-insert.md`
- `spec/tests.md`
- `review/README.md`

## What Landed

### 1. Metadata now persists `inserted_since_rebuild`

`MetadataPage` gains a persisted `u64 inserted_since_rebuild` field.

The codec and page-roundtrip coverage were updated so the on-disk metadata shape
now carries:

- graph shape and entry metadata
- tail-page metadata
- the inserted-since-rebuild counter

Bulk build initializes that field to `0`, so newly built or rebuilt indexes
start from a clean drift baseline.

### 2. Successful live inserts now bump the counter exactly once

The live insert path now increments `metadata.inserted_since_rebuild` only when a
new element tuple is actually appended.

That means:

- first insert into an empty index yields `inserted_since_rebuild = 1`
- non-empty graph-connected inserts increment once per newly appended node
- duplicate coalescing does **not** increment the counter

This checkpoint does not change the existing duplicate semantics or graph
selection logic; it only makes the drift counter reflect real newly inserted
graph nodes.

### 3. Metadata write ordering stays compatible with ADR-026

This slice changes one operational detail:

- successful live inserts now always end with a metadata-page write phase,
  because the drift counter is metadata-resident

The ordering contract is still:

1. traversal / planning first
2. append and any backlink rewrites on data pages
3. metadata last

So the checkpoint does **not** introduce mixed data-page + metadata-page write
lock overlap. It keeps the existing “metadata last” rule from ADR-026.

### 4. The admin snapshot now exposes real drift observability

`tqhnsw_index_admin_snapshot(regclass)` now reports:

- `block_count`
- `total_live_nodes`
- `inserted_since_rebuild`
- `insert_drift_fraction`
- `relation_ef_search`
- `session_ef_search`
- `effective_ef_search`
- `effective_source`
- `planner_scan_enabled`

`insert_drift_fraction` is derived as:

- `0.0` for an empty index
- otherwise `inserted_since_rebuild / total_live_nodes`

This moves the admin surface from “scaffolding with NULL drift field” to a real
FR-016-AC-4 observability surface.

### 5. Coverage now locks in both counter semantics and SQL visibility

New regression coverage verifies:

- build metadata initializes the counter to `0`
- empty-index first insert reports `1 / 1 = 1.0`
- non-empty live inserts advance the counter and total live-node count together
- duplicate-coalesced inserts do not advance the counter
- non-`tqhnsw` indexes are rejected by the admin snapshot surface

Existing insert shape, promotion, forward-link, backlink, overflow-pruning, and
empty-index tests remain in place and passed unchanged.

### 6. Tracking/spec docs now reflect the landed state

The docs now record that:

- A5 is effectively at the drift-accounting checkpoint with only concurrency
  hardening left
- `FR-016` now treats the admin snapshot as satisfying the observability portion
  of insert drift measurement
- `TC-133` explicitly targets `tqhnsw_index_admin_snapshot(regclass)`

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All passed on this checkpoint.

## New / Updated Coverage

- `test_tqhnsw_index_admin_snapshot_tracks_insert_drift`
- `test_tqhnsw_index_admin_snapshot_counts_empty_first_insert`
- `test_tqhnsw_index_admin_snapshot_rejects_non_tqhnsw_index`
- metadata roundtrip unit tests in `src/am/page.rs`
- metadata property coverage in `tests/proptest_page.rs`

## Review Focus

- Is persisting `inserted_since_rebuild` in `MetadataPage` the right narrow
  place for this counter, or is there a better v0.1 home before vacuum work
  begins?
- Is “increment only on real appended element tuples” the right contract for
  duplicate-coalesced inserts and empty-index first insert behavior?
- Is the new “successful live inserts always finish with a metadata write phase”
  still narrow enough under ADR-026, or does it expose a lock/ordering concern
  that should be addressed before the final concurrency-hardening slice?
