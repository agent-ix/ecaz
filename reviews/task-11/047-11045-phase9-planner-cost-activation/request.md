# Review Request: Planner Cost Activation (Phase 9)

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `src/am/ec_diskann/mod.rs`
- `src/am/ec_diskann/cost.rs`
- `src/am/ec_diskann/routine.rs`

## What this packet is

This is the Phase 9 slice for `ec_diskann`: it replaces the old
`disable_cost` planner shim with a live `amcostestimate` callback and
adds pg coverage that proves the planner now behaves correctly at the
two load-bearing boundaries:

1. empty indexes stay planner-gated
2. small ordered queries still prefer seqscan
3. large ordered queries naturally pick the `ec_diskann` index

Before this packet, `ec_diskann` had a working runtime scan path but the
planner callback still returned `pg_sys::disable_cost`, so Postgres would
never choose it without `enable_seqscan = off`. After this packet, the
planner can cost and select the access method on its own.

## Why this slice

Phase 6 and Phase 7/8 already closed the runtime contract:

- build persists a real Vamana index
- scan executes ordered retrieval with grouped-PQ prefilter plus exact
  heap rerank
- insert and vacuum keep the persisted graph live enough to scan

At that point the remaining planner blocker was self-inflicted: the
callback still advertised an infinite cost no matter what the index
looked like. Replacing that shim is the narrowest Phase 9 closeout.

## What changed

### `cost.rs`

Added a dedicated `src/am/ec_diskann/cost.rs` module with:

- **`ec_diskann_amcostestimate(...)`**
- **`compute_amcostestimate(index_relation)`**

The new callback mirrors the live `ec_hnsw` shape:

1. open the index relation with `NoLock`
2. read reloptions and block count
3. keep empty indexes gated with
   `block_count <= FIRST_DATA_BLOCK_NUMBER`
4. read persisted metadata for dimensions
5. delegate to the shared FR-020-style estimator in
   `am::common::cost`

The DiskANN-specific inputs are:

- `m = relation_options.graph_degree`
- `ef_search = relation_options.list_size`
- `dimensions = metadata.dimensions`
- `tree_height = 1.0`

That `tree_height = 1.0` choice is intentional: V0 `ec_diskann` is a
single-layer Vamana graph, so the planner should model one graph-entry
phase rather than the HNSW-style multi-layer descent.

### `mod.rs`

- added `mod cost;`
- flipped `ECDISKANN_PLANNER_SCAN_ENABLED` to `true`

The constant is not the functional switch by itself, but Phase 9 is now
actually live, so the module-level planner flag should match reality.

### `routine.rs`

`build_ec_diskann_routine()` now wires:

- **`amroutine.amcostestimate = Some(cost::ec_diskann_amcostestimate)`**

and removes the old local stub that returned `pg_sys::disable_cost`.

The test module also now has a small EXPLAIN helper seam:

- **`explain_text(...)`**
- **`explain_ordered_diskann_ids(...)`**
- **`explain_plan_uses_index(...)`**
- **`diskann_large_query_array()`**

This keeps the new planner pg tests narrow instead of repeating the
EXPLAIN row-decoding boilerplate three times.

## pg coverage

Added:

- **`test_ec_diskann_empty_index_remains_planner_gated`**
  proves an index with only block 0 metadata still does not win plan
  selection
- **`test_ec_diskann_planner_prefers_seqscan_for_small_tables`**
  proves a 50-row ordered query still stays on seqscan
- **`test_ec_diskann_planner_chooses_index_scan_for_large_table`**
  builds a 10K-row 64-dim fixture and proves the planner naturally
  selects `ec_diskann_large_plan_idx`

Retained:

- **`test_ec_diskann_sql_ordered_index_scan_executes`**
  still forces the runtime scan path and now uses the shared EXPLAIN
  helper

## Boundary after this packet

`ec_diskann` now has:

- build wiring
- runtime ordered scan wiring
- live insert
- live vacuum
- planner cost activation

So the main task-17 AM callback surface is now wired end-to-end. The
remaining work on this branch is follow-up hygiene, review handling, and
any later non-V0 planner/measurement refinement, not another missing AM
callback slice.

## Tests

New coverage:

- **`test_ec_diskann_empty_index_remains_planner_gated`**
- **`test_ec_diskann_planner_prefers_seqscan_for_small_tables`**
- **`test_ec_diskann_planner_chooses_index_scan_for_large_table`**

Retained relevant coverage:

- **`test_ec_diskann_sql_ordered_index_scan_executes`**
- full `ec_diskann` unit + pg-test surface
- full repo `cargo test`
- full pg17 script

## Verification

```text
cargo fmt -- src/am/ec_diskann/mod.rs src/am/ec_diskann/cost.rs src/am/ec_diskann/routine.rs
cargo build --lib
cargo clippy --lib --no-deps
cargo test --lib ec_diskann
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Observed:

- `cargo fmt -- src/am/ec_diskann/mod.rs src/am/ec_diskann/cost.rs src/am/ec_diskann/routine.rs`
  — passed
- `cargo build --lib` — passed
- `cargo clippy --lib --no-deps` — passed with only the known baseline
  `unnecessary_sort_by` warnings in untouched `reader.rs`, `scan.rs`,
  and `vamana.rs`
- `cargo test --lib ec_diskann` — passed with `143 passed`, `0 failed`
- `cargo test` — passed
- `bash scripts/run_pgrx_pg17_test.sh` — passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  — still fails only on the untouched baseline:
  - existing `reader.rs`, `scan.rs`, and `vamana.rs` sort warnings
  - existing `scan.rs` test-only `unnecessary_cast` /
    `needless_borrows_for_generic_args`
  - existing `vacuum.rs` test-only `needless_range_loop`

## Reviewer notes

- **Empty-index gating uses data-page count, not raw page count.** The
  callback gates on `block_count <= FIRST_DATA_BLOCK_NUMBER`, so block 0
  metadata alone does not accidentally make an empty index look usable.
- **The model is intentionally single-layer.** DiskANN does not have an
  HNSW-style `max_level`; using `tree_height = 1.0` keeps the estimator
  aligned with the actual runtime shape.
- **This packet only activates costing, not a new runtime path.** The
  ordered scan implementation already existed; this slice just makes the
  planner willing to choose it.
- **The large-table proof is natural selection, not forced selection.**
  The new 10K-row pg test does not disable seqscan.

## Not doing in this packet

- **Any strict-clippy baseline cleanup outside touched files**
- **Any new snapshot/admin SQL surface for `ec_diskann`**
- **Any work outside `src/am/ec_diskann/`, `review/`, or packet docs**
