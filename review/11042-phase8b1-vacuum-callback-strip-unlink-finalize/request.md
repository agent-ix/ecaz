# Review Request: Vacuum Callback Strip + Unlink + Finalize (Phase 8B-1)

Branch: `adr034-diskann-rebased`
Author: coder-2
Target: `src/am/ec_diskann/insert.rs`,
`src/am/ec_diskann/routine.rs`

## What this packet is

This is the first pgrx-side `ec_diskann` vacuum slice after the Phase 7
duplicate-bind work.

It replaces the `ambulkdelete` / `amvacuumcleanup` panics with a real
callback path that now does four load-bearing things:

1. **reports real vacuum stats** for empty and non-empty indexes
2. **strips dead heap tids** from node ownership, including duplicate
   overflow chains
3. **unlinks finalized dead node tids** from live neighbor arrays
4. **tombstones fully-dead nodes** and flips
   `needs_medoid_refresh = true` when the entry point dies

Before this slice, any vacuum entry into `ec_diskann` still errored.
After this slice, vacuum can clean the V0 graph enough to keep scan
semantics correct for dead rows and duplicate-bound nodes.

## Why this slice

Phase 7 introduced a new constraint for Phase 8B:

- a live node may now own multiple heap rows
- scan expansion still assumes every live node has a real
  `primary_heaptid`

That means vacuum could not just strip a dead primary and leave an
"overflow-only" node behind. The first callback slice therefore had to
do both:

1. the pgrx vacuum callback wiring itself
2. duplicate-aware heap-tid compaction/promotion in the node owner

That is the minimum safe callback boundary for V0.

## What changed

### `insert.rs`

Added the new helper:

- **`vacuum_bound_heap_rows(...)`**

This reuses the Phase 7 overflow-tuple codec and makes pass 1
duplicate-safe:

1. strip dead primary heap tids
2. filter dead heap tids out of every overflow tuple owned by the node
3. if the primary died but an overflow heap tid survived, promote the
   first surviving overflow heap tid into `primary_heaptid`
4. compact the remaining live overflow heap tids back into the existing
   fixed-capacity overflow tuples
5. clear `has_overflow_heaptids` when no overflow heap tids remain

Without this helper, vacuum would have left nodes in an unsupported
state for the current scan path.

### `routine.rs`

Replaced both vacuum stubs:

- **`ec_diskann_ambulkdelete`**
- **`ec_diskann_amvacuumcleanup`**

The new callback path is intentionally narrow and reuses Phase 8A's
pure-Rust tuple primitives without editing `vacuum.rs`.

#### Stats path

- **`ec_diskann_noop_vacuum_stats(...)`** now returns real page and live
  tuple counts instead of panicking.

#### Bulkdelete path

- **`run_diskann_bulkdelete(...)`** now:
  1. materializes the persisted chain
  2. runs pass 1 dead-heap-tid stripping through
     `insert::vacuum_bound_heap_rows(...)`
  3. records the fully-dead node set
  4. runs pass 2 unlink-only neighbor repair through
     `vacuum::repair_neighbors(...)`
  5. runs pass 3 finalization through `vacuum::mark_deleted(...)`
  6. writes only changed tuples back to disk page-by-page with
     `GenericXLog`
  7. sets `needs_medoid_refresh = true` if the finalized tuple was the
     current metadata entry point
  8. updates `num_index_tuples`, `num_pages`, and `tuples_removed`

#### Rewrite safety

The page writer does **not** blindly overwrite tuples. It first checks
that the reopened page still matches the materialized snapshot bytes for
every tuple it intends to patch. If any planned tuple has drifted, the
rewrite errors instead of silently clobbering newer state.

That is a deliberate conservative boundary for this slice.

## Boundary after this packet

`ec_diskann` vacuum now supports:

- benign noop stats
- empty-index cleanup
- duplicate-aware pass 1 heap-tid stripping
- duplicate-primary promotion out of overflow
- pass 2 unlink-only neighbor cleanup for finalized dead nodes
- pass 3 fully-dead tombstoning
- vacuum ownership of `needs_medoid_refresh`

`ec_diskann` vacuum still does **not** support:

- fill/replacement planning for newly-freed neighbor slots
- bounded repair replan after tuple drift
- concurrent insert/vacuum repair retries on the same tuple
- any cold rerank chain work (still correctly absent in V0)

So this is a real vertical callback slice, but it is intentionally the
**unlink/finalize boundary**, not the full ADR-047 repair-replan closeout.

## Tests

New pg coverage in `routine.rs`:

- **`test_ec_diskann_vacuum_noop_stats_on_empty_index`**
  proves the callback pair is benign on an empty index and reports real
  stats
- **`test_ec_diskann_vacuum_promotes_duplicate_overflow_primary`**
  proves vacuum removes a dead duplicate primary, promotes the surviving
  overflow heap tid into `primary_heaptid`, and preserves runtime scan
  correctness
- **`test_ec_diskann_vacuum_unlinks_and_tombstones_dead_node`**
  proves pass 2 removes dead neighbor refs from live nodes and pass 3
  tombstones the fully-dead node
- **`test_ec_diskann_vacuum_sets_medoid_refresh_flag`**
  proves vacuum owns the monotonic metadata bit when the entry-point row
  dies

Retained coverage:

- full `ec_diskann` unit + pg-test surface
- full repo `cargo test`
- full pg17 script

## Verification

```text
cargo fmt -- src/am/ec_diskann/insert.rs src/am/ec_diskann/routine.rs
cargo build --lib
cargo clippy --lib --no-deps
cargo test --lib ec_diskann
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Observed:

- `cargo fmt -- src/am/ec_diskann/insert.rs src/am/ec_diskann/routine.rs`
  — passed
- `cargo build --lib` — passed
- `cargo clippy --lib --no-deps` — passed with only the known baseline
  `unnecessary_sort_by` warnings in untouched `reader.rs`, `scan.rs`,
  and `vamana.rs`
- `cargo test --lib ec_diskann` — passed with `138 passed`, `0 failed`
- `cargo test` — passed
- `bash scripts/run_pgrx_pg17_test.sh` — passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  — still fails only in untouched baseline code/tests:
  - existing `reader.rs`, `scan.rs`, and `vamana.rs` sort warnings
  - existing `scan.rs` test-only warnings
  - existing `vacuum.rs` test-only warning

## Reviewer notes

- **Duplicate-bound nodes are now vacuum-safe.** Pass 1 will not leave a
  live node with `primary_heaptid = INVALID` and only overflow heap
  tids.
- **This slice is unlink-only in pass 2.** Dead neighbors are removed,
  but newly-free slots are not refilled yet.
- **Current rewrite staleness policy is conservative.** If a target
  tuple drifted after materialization, the write path errors instead of
  trying to replan in place.
- **`needs_medoid_refresh` stays vacuum-only and monotonic.** Insert
  still does not touch it.
- **No `vacuum.rs` edit was needed.** The callback layer reuses the
  Phase 8A pure-Rust primitives and keeps this slice inside
  `routine.rs` + the duplicate overflow owner helper.

## Not doing in this packet

- **Neighbor refill / candidate planning**
- **Repair replan cap / retry loop**
- **Any cold rerank payload handling**
- **Any change outside `src/am/ec_diskann/`, `review/`, or packet docs**
