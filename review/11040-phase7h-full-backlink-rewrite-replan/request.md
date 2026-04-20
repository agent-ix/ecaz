# Review Request: Full Backlink Rewrite Replan (Phase 7H)

Branch: `adr034-diskann-rebased`
Author: coder-2
Target: `src/am/ec_diskann/insert.rs`,
`src/am/ec_diskann/routine.rs`

## What this packet is

This is the next narrow insert slice after 11039's metadata
bookkeeping.

It lands the first full-target backlink rewrite path for non-empty
unique inserts:

1. plan an exact-pruned replacement slice for a full backlink target
2. reopen that target under write lock and either apply the rewrite or
   request a retry if the slice drifted
3. retry from fresh persisted state up to a bounded pass cap

Free-capacity backlinks still take the cheap path. This packet only
changes the full-target case.

## Why this slice

ADR-046 reviewer feedback on 11002 called out two things explicitly:

- full-target rewrite should replan against reopened state instead of
  trusting a stale snapshot
- the retry loop needs a hard cap (`MAX_BACKLINK_REPLAN_PASSES = 3`)

That is a clean seam to land before overflow-heaptid work or
entry-point maintenance. It also closes the main correctness gap left
after 11038: new nodes could append and install free-slot backlinks, but
they could not yet displace a worse backlink occupant from a full target
slice.

## What changed

### `insert.rs`

Added:

- **`MAX_BACKLINK_REPLAN_PASSES: usize = 3`**
- **`BacklinkMutation`**
- **`BacklinkMutationKind::{InsertIfFree, RewriteFullSlice { ... }}`**
- **`BacklinkMutationOutcome::{NoChange, Changed, RetryReplan}`**
- **`plan_backlink_mutation(...) -> Result<Option<BacklinkMutation>, String>`**
- **`apply_backlink_mutation(...) -> BacklinkMutationOutcome`**
- **`apply_backlink_mutations(...) -> Result<Vec<ItemPointer>, String>`**

`plan_backlink_mutation(...)` does:

1. no-op when the target already contains the new node
2. keep the existing cheap path when the target still has an invalid
   slot
3. for a full slice, exact-prune `existing candidates + new candidate`
   against the target source vector using
   `select_insert_forward_neighbors(...)`
4. emit a `RewriteFullSlice` plan only when the new node survives prune

`apply_backlink_mutation(...)` enforces the write-window stale check:

1. if the target now already contains the new node, do nothing
2. if a free slot reopened, fall back to the cheap insert
3. if the reopened slice no longer matches the planned
   `(neighbors, neighbor_count)` snapshot, return `RetryReplan`
4. otherwise write the planned replacement neighbors and count

`apply_backlink_mutations(...)` groups mutations by block, reopens each
target page under `BUFFER_LOCK_EXCLUSIVE`, WAL-registers the page, and
returns the subset of target tids that requested replanning.

### `routine.rs`

`ec_diskann_aminsert` now calls:

- **`install_backlinks_with_replan(...)`**

instead of the old free-capacity-only helper.

`install_backlinks_with_replan(...)`:

1. sorts and deduplicates the planned backlink targets
2. replans from fresh persisted state on every pass
3. applies the page writes for that pass
4. retries only the tids that came back as `RetryReplan`
5. errors if any targets still need replanning after
   `MAX_BACKLINK_REPLAN_PASSES`

The planning side lives in:

- **`plan_backlink_mutations(...)`**

That helper materializes a fresh chain snapshot through
`scan_state::materialize_chain_from_index(...)`, fetches the target and
current-neighbor source vectors from the heap, and builds the exact
rewrite plans outside the write window.

That keeps the write window page-local: the lock-holding path only
reopens the tuple, checks whether the planned slice is still current,
and writes the replacement bytes if so.

### pg test

Added:

- **`test_ec_diskann_full_backlink_rewrite_keeps_insert_reachable`**

The test intentionally uses a 5-D fixture because insert-side exact
distance is `(-ip).max(0.0)`: positive inner products collapse to zero,
so a naive "slightly better dot product" fixture does not force a unique
rewrite target.

The final fixture does:

1. insert one seed node
2. insert four nodes that fill the seed's backlink slice
3. materialize the persisted graph and prove the seed slice is full
4. insert a sixth node whose source vector is positive against the seed
   and negative against the filler nodes
5. prove the seed slice changed and now includes the sixth node
6. prove the sixth node still keeps the seed as a forward neighbor
7. force runtime execution through `ec_diskann` and verify
   `SELECT ... ORDER BY ... LIMIT 1` returns row `6`

The post-rewrite slice is **not** asserted to stay at full capacity.
That was an incorrect assumption during development: once the full slice
is reopened, exact prune is allowed to keep fewer than `graph_degree`
neighbors.

## Boundary after this packet

Non-empty unique insert now:

- derives persisted payload
- rejects true duplicates
- plans exact forward neighbors
- appends the new node
- writes free-capacity backlinks
- rewrites full backlink slices with bounded replan on drift
- increments `inserted_since_rebuild`

Insert still does **not**:

- grow overflow-heaptid chains
- maintain or repair the entry point
- update `needs_medoid_refresh`

## Tests

New coverage:

- **`in_013_plan_backlink_mutation_rewrites_full_slice_for_kept_candidate`**
- **`in_014_apply_backlink_mutation_requests_retry_for_stale_full_slice`**
- **`in_015_apply_backlink_mutation_rewrites_full_slice_after_replan`**
- **`test_ec_diskann_full_backlink_rewrite_keeps_insert_reachable`**

Retained coverage:

- full `ec_diskann` pure-Rust and pg test suite
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
- `cargo clippy --lib --no-deps` — passed with the known baseline
  `unnecessary_sort_by` warnings in untouched `reader.rs`, `scan.rs`,
  and `vamana.rs`
- `cargo test --lib ec_diskann` — passed with `133 passed`, `0 failed`
- `cargo test` — passed
- `bash scripts/run_pgrx_pg17_test.sh` — passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  — still fails only in untouched baseline code/tests:
  - existing `reader.rs`, `scan.rs`, and `vamana.rs` sort warnings
  - existing `scan.rs` test-only warnings
  - existing `vacuum.rs` test-only warning

## Reviewer notes

- **The write window is intentionally narrow.** Heap fetches and exact
  pruning happen during the read-only planning pass, not while holding
  the target page lock.
- **Retry state is target-local.** The retry loop only carries the
  pending target tids forward and replans from a fresh materialized
  chain each pass.
- **Full-slice rewrite can shrink the live prefix.** That is deliberate
  and follows the existing exact prune contract.
- **The new pg test is fixture-sensitive by design.** The 5-D shape is
  there to avoid the zero-clamp tie behavior in
  `source_inner_product_distance(...)`.

## Not doing in this packet

- **Overflow-heaptid growth**
- **Entry-point maintenance**
- **Any insert-side write to `needs_medoid_refresh`**
