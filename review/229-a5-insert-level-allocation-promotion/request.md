# Review Request: A5 Insert Level Allocation and Metadata Promotion

## Context

Branch:
- `main`

Task / roadmap inputs:
- `plan/tasks/06-graph-insert.md`
- `review/199-aminsert-graph-aware-insertion-roadmap/request.md`
- `plan/status.md`

A4 is closed on `main`, B1 SIMD is merged, and A5 is now the next runtime lane.
This checkpoint intentionally takes the first narrow insert slice from review 199
before any graph-edge mutation:

1. random level assignment during live insert
2. pre-sized neighbor tuple allocation based on `(level, m)`
3. entry-point / max-level promotion when the new node outranks the current entry
4. regression coverage for tuple shape and metadata promotion

No forward links, backlinks, or neighbor shrinking land in this packet.

## Scope

- `src/am/insert.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/06-graph-insert.md`

## What Landed

### 1. Live inserts now assign a real HNSW level

`aminsert` no longer hardcodes `level = 0`.

The new helper path:

- derives a deterministic pseudo-random sample from the persisted index seed plus
  the heap TID
- maps that sample through the HNSW exponential level formula
- caps the result to the highest level that still fits the current append model's
  neighbor+element tuple pair on one fresh data page

This keeps the checkpoint deterministic for tests while still giving live inserts a
real level distribution instead of the old disconnected level-0-only behavior.

### 2. Neighbor tuples now allocate the full slot shape for the inserted level

Live inserts now write:

- `count = neighbor_slots(level, m)`
- `tids = vec![INVALID; neighbor_slots(level, m)]`

instead of the old empty payload.

That aligns live insert storage with the existing build path and with the scan/runtime
layer-slicing helpers that expect stable `2M` / `M` slot boundaries.

### 3. Metadata promotion now tracks inserted higher-level nodes

After append, `aminsert` now repairs/promotes metadata under the metadata-page lock when:

- `entry_point` is invalid, or
- the new node's level exceeds `metadata.max_level`

In both cases the metadata update writes:

- `entry_point = new_element_tid`
- `max_level = new_element_level`

This keeps the metadata invariant that the entry point is a live element at the
recorded `max_level`.

### 4. Task docs now match current project state

`plan/tasks/06-graph-insert.md` no longer says A5 is blocked on older traversal work.
It now records that A5 is in progress on `main` and that this first checkpoint covers
level assignment, tuple allocation, and metadata promotion.

## Locking Decision

This packet does **not** introduce multi-page neighbor/backlink mutation, so it does
not need the future backlink lock-ordering ADR yet.

The lock scope remains:

1. duplicate scan outside the metadata EXCLUSIVE lock
2. append under the target data-page EXCLUSIVE buffer lock
3. metadata promotion/repair afterward under the metadata-page EXCLUSIVE lock only when needed

No insert in this slice holds multiple data-page locks at once.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All passed on this checkpoint.

## New / Updated Coverage

- `test_tqhnsw_empty_index_insert_initializes_shape_metadata`
- `test_tqhnsw_empty_index_reuses_initialized_metadata`
- `test_tqhnsw_insert_repairs_invalid_entry_point_after_shape_init`
- `test_tqhnsw_insert_neighbor_tuple_sizing_matches_levels`
- `test_tqhnsw_insert_promotes_entry_point_on_level_up`
- `test_tqhnsw_insert_reuses_new_tail_page_after_rollover`

## Review Focus

- Is the deterministic pseudo-random level assignment acceptable for this live-insert slice?
- Is the page-fit cap for the current append model the right boundary until multi-page insert lands?
- Does the metadata repair/promotion rule preserve the intended `entry_point` / `max_level` invariant?
- Is the checkpoint still narrow enough, or should the next slice go straight to greedy descent + layer-0 forward links?
