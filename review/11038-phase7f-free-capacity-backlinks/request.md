# Review Request: Free-Capacity Backlinks (Phase 7F)

Branch: `adr034-diskann-rebased`
Author: coder-2
Target: `src/am/ec_diskann/insert.rs`,
`src/am/ec_diskann/routine.rs`

## What this packet is

This is the next narrow non-empty insert slice after 11037's
forward-link append.

It lands the smallest backlink write that makes an appended node
runtime-reachable without taking on the full ADR-046 rewrite/retry
surface:

1. append the new live node with its planned forward neighbors
2. try to add the reverse edge into each forward target if that target
   still has a free neighbor slot
3. prove the persisted backlink exists and that runtime ordered scan can
   now reach the inserted row

It still does **not** rewrite full neighbor slices when a target is
full, retry stale plans, or update non-empty insert metadata counters.

## Why this slice

11037 made distinct inserts durable and duplicate-visible, but the new
node could still be unreachable from the entry point because existing
nodes were never patched.

The next safe boundary is "free-capacity backlinks only":

- no slice rewrite logic yet
- no stale-target retry loop yet
- no metadata maintenance yet
- but enough graph integration to make the appended node discoverable
  through the live scan path when its chosen forward targets have spare
  degree

That turns the append-only boundary into a real reachability boundary
without pretending Phase 7 is complete.

## What changed

### `insert.rs`

Added:

- **`insert_backlink_if_free(...)`**
- **`add_backlinks_if_free(...)`**
- private helpers **`sort_and_dedup_backlink_targets(...)`** and
  **`page_tuple_location(...)`**

`insert_backlink_if_free(...)` mutates an already-decoded
`VamanaNodeTuple` in memory:

1. reject `INVALID` backlink tids
2. reject duplicates
3. fill the first `INVALID` neighbor slot
4. extend `neighbor_count` only when the new edge lands beyond the
   previous live prefix

`add_backlinks_if_free(...)` is the pgrx write helper. It:

1. sorts and deduplicates backlink targets by physical `(block, offset)`
   order
2. opens one target block at a time under `BUFFER_LOCK_EXCLUSIVE`
3. registers that page with `GenericXLog`
4. decodes only live node tuples using the current metadata-derived
   binary/search-code lengths
5. writes the updated tuple bytes back in place only when a free slot
   was actually consumed

New unit tests:

- **IN-011** proves the helper uses the first available slot
- **IN-012** proves duplicate and full-target cases are rejected

### `routine.rs`

`ec_diskann_aminsert` now extends the 11037 unique-insert flow:

1. derive the persisted payload
2. reject exact duplicates
3. plan exact forward neighbors
4. append the new live node
5. call `insert::add_backlinks_if_free(...)` for those forward targets

The callback still does **not** rewrite full targets or retry if a
target drifted. Any such case remains deferred to a later Phase 7 slice.

### strengthened pg proof

`test_ec_diskann_unique_insert_is_scan_reachable` now replaces the old
append-only smoke:

1. bootstrap a one-row index
2. insert a distinct second row
3. materialize the persisted chain from the index relation
4. locate the node tids for the seed row and the inserted row
5. assert the inserted node kept the seed node as a forward neighbor
6. assert the seed node received a backlink to the inserted node
7. force runtime ordered execution through `ec_diskann`
8. assert the inserted row ranks first for its own query vector

That makes this slice's contract concrete: not just "bytes were
written", but "the graph is now reachable enough for runtime scan to
return the inserted row".

## Boundary after this packet

Non-empty unique insert now:

- derives persisted payload
- rejects true duplicates
- plans exact forward neighbors
- appends the new node tuple
- backfills reverse edges into forward targets when those targets still
  have free neighbor capacity
- makes the common two-node reachability case visible to runtime scan

Non-empty unique insert still does **not**:

- rewrite a full target's neighbor slice
- retry against a changed target snapshot
- grow overflow-heaptid chains
- update `inserted_since_rebuild`
- update `needs_medoid_refresh`
- promote or repair the entry point

## Tests

New coverage:

- **IN-011** first-free-slot backlink insertion
- **IN-012** duplicate/full backlink rejection
- **`test_ec_diskann_unique_insert_is_scan_reachable`** persisted
  backlink + runtime reachability proof

Retained coverage:

- **`test_ec_diskann_duplicate_insert_hits_duplicate_boundary`**
- **`test_ec_diskann_empty_index_bootstrap_insert_executes`**
- **`test_ec_diskann_duplicate_after_append_hits_boundary`**
- full `ec_diskann` pure-Rust and pg test suite

## Verification

```text
cargo build --lib
cargo clippy --lib --no-deps
cargo test --lib ec_diskann
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Observed:

- `cargo build --lib` — passed
- `cargo clippy --lib --no-deps` — passed with the known baseline
  `unnecessary_sort_by` warnings in untouched `reader.rs`, `scan.rs`,
  and `vamana.rs`
- `cargo test --lib ec_diskann` — passed with `129 passed`, `0 failed`
- `cargo test` — passed
- `bash scripts/run_pgrx_pg17_test.sh` — passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  — still fails only in untouched baseline code/tests:
  - existing `reader.rs`, `scan.rs`, and `vamana.rs` sort warnings
  - existing `scan.rs` test-only warnings
  - existing `vacuum.rs` test-only warning

The touched Phase 7F files (`insert.rs`, `routine.rs`) were also checked
with file-scoped `rustfmt`; no additional formatting-only delta was
produced.

## Reviewer notes

- **This is a free-slot-only backlink slice.** Reviewers should assess
  the in-place tuple rewrite helper on its own terms, not as a full
  ADR-046 completion.
- **Physical-order batching is intentional.** Backlink targets are
  sorted/deduped by `(block, offset)` before the write loop so one block
  lock can service multiple targets.
- **Reachability is now part of the proof surface.** The strengthened pg
  test checks both persisted adjacency and runtime scan output.

## Not doing in this packet

- **Full-slice backlink rewrite when a target is full**
- **Retry/replan loop for stale or drifted targets**
- **Metadata counter maintenance**
- **Overflow-heaptid chain growth**
- **Entry-point maintenance**
