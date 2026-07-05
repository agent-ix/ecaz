# Review Request: Unique Insert Forward-Planning Boundary (Phase 7D)

Branch: `adr034-diskann-rebased`
Author: coder-2
Target: `src/am/ec_diskann/insert.rs`,
`src/am/ec_diskann/routine.rs`

## What this packet is

This is the next narrow non-empty insert slice after 11035's exact
duplicate-probe boundary.

It still does **not** append a new Vamana node or rewrite backlinks.
What it does land is the read-only planning work that has to exist
before those writes are safe:

1. discover unique-insert candidates from the persisted graph using the
   same scan-side greedy helper shape
2. materialize exact heap vectors for those candidates outside any data
   page write window
3. run Vamana `RobustPrune` over those exact vectors to choose the new
   node's forward neighbors
4. stop at a new explicit boundary message for the still-unimplemented
   append/backlink phase

So 11035 answered "is this a true duplicate?"; 11036 answers "if it is
not a duplicate, which existing nodes would the new node link to?"

## Why this slice

ADR-046's ordered write phase depends on one read-only invariant:
everything needed for `RobustPrune` under a future page lock must
already be materialized.

This packet freezes that seam without mixing in:

- new-node append page allocation
- ordered backlink rewrites
- stale-target retry
- `inserted_since_rebuild` metadata writes

That keeps the slice reviewable while still moving non-empty insert
past the generic panic boundary.

## What changed

### `insert.rs`

Added:

- **`ForwardNeighborCandidate { tid, source_vector }`**
- **`select_insert_forward_neighbors(source, candidates, alpha, max_degree)`**

`select_insert_forward_neighbors` is a pure-Rust helper that:

1. validates the exact source-vector dimensions
2. computes source-to-candidate exact distances with the same
   `max(0, -ip)` wrapper build uses
3. computes candidate-to-candidate exact distances once
4. feeds those exact distances into `vamana::robust_prune`
5. returns the selected existing node TIDs in prune order

This is the exact forward-neighbor selector the later append/write slice
will reuse once it starts persisting the new node and applying
backlinks.

New unit tests:

- **IN-008** proves exact-vector alpha prune keeps the expected two
  orthogonal neighbors and drops the dominated third candidate
- **IN-009** proves dimension mismatch is rejected

### `routine.rs`

`ec_diskann_aminsert` now has a real unique-insert planning path after
the 11035 duplicate check:

1. resolve the entry point with `scan::resolve_entry_point`
2. build the grouped-PQ LUT from persisted codebooks with
   `build_grouped_pq_lut_from_persisted`
3. reuse `scan::vamana_scan_with` for read-only candidate discovery,
   with:
   - `list_size = rerank_budget = top_k = metadata.build_list_size_l`
   - grouped-PQ search-code prefilter
   - exact heap rerank against the raw insert vector
4. fetch each exact candidate's heap `ecvector` into a
   `ForwardNeighborCandidate`
5. call `select_insert_forward_neighbors(...)`
6. raise the new boundary:

`ec_diskann unique insert append/backlink writes are not yet implemented (task 17 phase 7): planned N forward neighbors from entry (...)`

Also added:

- **`fetch_heap_source_vector(...)`** so duplicate probe, unique insert
  planning, and exact rerank share one heap-row extraction helper

Updated pg test:

- **`test_ec_diskann_unique_insert_hits_forward_planning_boundary`**
  now proves a distinct second insert gets past duplicate probing,
  plans one forward neighbor on the bootstrap one-node graph, and then
  stops at the new append/backlink boundary

The duplicate-specific boundary from 11035 remains intact.

## Candidate-discovery shape

This packet intentionally reuses the scan-side traversal shell instead
of inventing a second greedy walker:

- candidate discovery is the same grouped-PQ traversal shape as scan
- exact candidate selection still comes from heap `ecvector` rows
- `RobustPrune` sees only exact vectors already materialized during the
  read-only phase

That matches the ADR-046 reviewer guidance: page-local write windows may
use only inputs gathered before the lock is taken.

## Tests

New coverage:

- **IN-008** exact-vector alpha prune selects the expected forward
  neighbors
- **IN-009** exact-vector forward planning rejects dimension mismatch
- **`test_ec_diskann_unique_insert_hits_forward_planning_boundary`**

Retained coverage:

- **`test_ec_diskann_duplicate_insert_hits_duplicate_boundary`**
- **`test_ec_diskann_empty_index_bootstrap_insert_executes`**
- full `ec_diskann` scan + build + tuple + vacuum primitive test suite

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
- `cargo clippy --lib --no-deps` — clean apart from the 8 pre-existing
  `unnecessary_sort_by` warnings in `reader.rs`, `scan.rs`, and
  `vamana.rs`
- `cargo test --lib ec_diskann` — passed with `123 passed`, `0 failed`
- `cargo test` — passed
- `bash scripts/run_pgrx_pg17_test.sh` — passed
  The repo script is not executable in this checkout, so it was invoked
  through `bash` rather than as a bare path.
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  — still fails only on the known baseline warnings in untouched files:
  - 8 existing lib warnings in `reader.rs`, `scan.rs`, and `vamana.rs`
  - existing test-only warnings in `scan.rs` and `vacuum.rs`
  This packet's one new strict-clippy complaint
  (`items_after_test_module` in `insert.rs`) was fixed before finalizing
  the slice.

## Reviewer notes

- **No write phase yet.** This packet does not append a node, rewrite
  neighbors, add backlinks, grow overflow chains, or bump
  `inserted_since_rebuild`.
- **No metadata ownership change.** Insert still does not touch
  `needs_medoid_refresh`.
- **No cold rerank payload.** Candidate rerank and exact neighbor
  planning both continue to fetch the heap `ecvector`; `rerank_tid`
  stays `INVALID`.
- **Unique insert is now a narrower boundary.** Distinct second inserts
  no longer stop at the generic non-empty panic; they stop only after
  the forward-neighbor plan is computed.

## Not doing in this packet

- **New-node append**
- **Ordered backlink writes**
- **Stale-target retry / `MAX_BACKLINK_REPLAN_PASSES`**
- **Duplicate bind write path**
- **Overflow-heaptid chain growth**
- **Non-empty insert metadata updates**
