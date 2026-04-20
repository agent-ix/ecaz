# Review Request: Vacuum Callback Fill-Only Repair (Phase 8B-2)

Branch: `adr034-diskann-rebased`
Author: coder-2
Target: `src/am/ec_diskann/routine.rs`

## What this packet is

This is the second pgrx-side `ec_diskann` vacuum slice after packet
`11042` established pass 1 strip, pass 2 unlink, and pass 3 finalize.

It advances pass 2 from **unlink-only** to **fill-only repair**:

1. dead neighbor TIDs are still removed first
2. affected live nodes are then replanned against the persisted graph
3. only newly-freed `INVALID` slots are refilled
4. already-live surviving neighbors are preserved

Before this slice, vacuum could only compact a live node's neighbor
array downward after a delete. After this slice, vacuum can also refill
reachable replacement neighbors into the freed slots.

## Why this slice

Packet `11042` made the graph scan-correct for dead rows, but it still
left a structural gap:

- deleting one node could permanently reduce the out-degree of another
  live node
- repeated deletes would monotonically thin the live graph

The next safe boundary was therefore:

1. keep the same materialize-mutate-rewrite callback structure
2. add replacement planning only for nodes that actually lost neighbors
3. stop short of full stale-drift replan / retry logic

That yields a real vertical improvement without pretending the ADR-047
retry boundary is done.

## What changed

### `routine.rs`

`run_diskann_bulkdelete(...)` now takes the heap relation and tracks the
subset of live nodes that lost dead neighbor refs during pass 2.

New helpers:

- **`resolve_vacuum_heap_relation(...)`**
- **`release_owned_vacuum_heap_relation(...)`**
- **`fill_vacuum_neighbor_slots(...)`**
- **`plan_vacuum_fill_candidates_for_target(...)`**

#### Fill-only repair flow

After the existing unlink pass builds `repair_target_tids`, the callback
now:

1. resolves the heap relation once for repair planning
2. allocates one heap slot plus reusable `VisitedState`
3. for each affected live target:
   - fetches the exact source vector from heap
   - gathers surviving live neighbors as fixed keep-candidates
   - rebuilds the grouped-PQ LUT from persisted codebooks
   - resolves a live entry point from the current mutated chain
   - runs `scan::vamana_scan_with(...)` over the current graph
   - exact-reranks candidate heap rows against the target source vector
   - unions scanned candidates with the surviving neighbors
   - runs `insert::select_insert_forward_neighbors(...)`
   - inserts only *new* accepted TIDs into currently-free slots via
     `insert::insert_backlink_if_free(...)`

This is intentionally **fill-only**:

- live surviving neighbors are not evicted here
- full-slice backlink rewrite is still not part of vacuum
- if no new acceptable candidate is discovered, the target simply stays
  underfilled

#### Heap / score semantics

The repair planner mirrors the Phase 6B scan-side scoring contract:

- grouped-PQ prefilter uses `-grouped_pq_score_f32(...)`
- exact rerank uses the raw negative inner-product distance from the
  heap source vector

That keeps vacuum repair consistent with the existing `ec_diskann`
ordered-scan path.

## Boundary after this packet

`ec_diskann` vacuum now supports:

- duplicate-safe pass 1 heap-tid stripping
- pass 2 dead-neighbor unlink
- pass 2 fill-only replacement planning for newly-freed slots
- pass 3 tombstoning of fully-dead nodes
- metadata `needs_medoid_refresh` ownership when the entry point dies

`ec_diskann` vacuum still does **not** support:

- full-slice repair rewrite when a repair target has no free slot
- bounded repair retry when tuple bytes drift before rewrite
- concurrent insert/vacuum replan loops on the same tuple
- any cold rerank chain work (still correctly absent in V0)

So this is the **fill-only repair boundary**, not the final ADR-047
closeout.

## Tests

New pg coverage in `routine.rs`:

- **`test_ec_diskann_vacuum_refills_broken_neighbor_slot`**
  proves a live node that loses a dead neighbor can regain a reachable
  replacement in the freed slot after vacuum

Test note:

- the new test rewrites one live target's neighbor slice on-disk before
  running vacuum, then searches for a reachable refill scenario using
  the real planner helper. That keeps the production code unchanged
  while avoiding accidental dependence on one specific build topology.

Retained coverage:

- full `ec_diskann` unit + pg-test surface
- full repo `cargo test`
- full pg17 script

## Verification

```text
cargo fmt -- src/am/ec_diskann/routine.rs
cargo build --lib
cargo clippy --lib --no-deps
cargo test --lib ec_diskann
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Observed:

- `cargo fmt -- src/am/ec_diskann/routine.rs` — passed
- `cargo build --lib` — passed
- `cargo clippy --lib --no-deps` — passed with only the known baseline
  `unnecessary_sort_by` warnings in untouched `reader.rs`, `scan.rs`,
  and `vamana.rs`
- `cargo test --lib ec_diskann` — passed with `139 passed`, `0 failed`
- `cargo test` — passed
- `bash scripts/run_pgrx_pg17_test.sh` — passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  — still fails only in untouched baseline code/tests:
  - existing `reader.rs`, `scan.rs`, and `vamana.rs` sort warnings
  - existing `scan.rs` test-only `unnecessary_cast` /
    `needless_borrows_for_generic_args`
  - existing `vacuum.rs` test-only `needless_range_loop`

## Reviewer notes

- **Pass 2 is now two-stage.** Dead refs are removed first, then only
  the nodes that actually lost neighbors are considered for refill.
- **Repair planning reads the current mutated chain, not the original
  snapshot.** That means refill candidates are chosen against the
  already-unlinked graph state for this vacuum pass.
- **This slice never rewrites a full live neighbor slice.** If a target
  has no free slot, vacuum does not attempt a full eviction/replan path
  here.
- **Tuple-byte drift is still a hard error.** The same conservative
  rewrite-safety policy from packet `11042` remains in force.
- **No new files outside `routine.rs` were needed.** The callback layer
  reuses persisted scan helpers plus the existing insert-side exact
  neighbor selector.

## Not doing in this packet

- **Full-slice vacuum repair rewrite**
- **Bounded repair retry / stale-drift replan**
- **Any change outside `src/am/ec_diskann/`, `review/`, or packet docs**
