# Review Request: Forward-Link Append Boundary (Phase 7E)

Branch: `adr034-diskann-rebased`
Author: coder-2
Target: `src/am/ec_diskann/reader.rs`,
`src/am/ec_diskann/insert.rs`,
`src/am/ec_diskann/routine.rs`

## What this packet is

This is the next narrow non-empty insert slice after 11036's
forward-neighbor planning boundary.

It lands the first real non-empty Vamana write:

1. append a new live node tuple carrying the planned forward neighbors
2. keep duplicate probing working after that append
3. keep scan-side live-node iteration working when grouped-codebook
   tuples sit before later appended nodes

It still does **not** rewrite backlinks, retry stale targets, or update
non-empty insert metadata counters.

## Why this slice

The persisted chain already stores grouped-codebook tuples after the
built node set. A naive append would put new nodes physically after
those codebooks, which breaks any reader that assumes "all nodes are
before the codebook tail".

This packet fixes that boundary first, then uses it to land the
smallest write phase that is actually durable:

- node-only iteration becomes tag-aware
- duplicate probing becomes tag-aware across the full chain
- unique insert appends only the new node tuple with forward links

That makes a second distinct insert persist successfully without
pretending backlinks or metadata maintenance are done.

## What changed

### `reader.rs`

Added:

- **`tuple_tag(tid)`**
- **`iter_node_tids()`**

`iter_live_tids()` now walks only node-tag tuples, so mixed-tag chains
skip grouped-codebook shards instead of trying to decode them as Vamana
nodes.

New unit tests:

- **RD-020** proves `iter_live_tids()` skips a grouped-codebook tuple
  between two node tuples
- **RD-021** proves `first_live_tid()` can skip a dead leading node plus
  an intervening codebook tuple and still find a later appended live
  node

### `insert.rs`

Changed:

- **`duplicate_candidate_tids_by_payload(...)`** now scans
  `reader.iter_node_tids()` across the full chain instead of stopping at
  `grouped_codebook_head`

Added:

- **`append_live_node(...)`**
- private helper **`append_live_node_payload(...)`**

`append_live_node(...)` encodes a new live `VamanaNodeTuple` from the
insert payload plus the already-planned forward-neighbor list, then:

1. reuses the last data block when there is enough free space
2. falls back to `P_NEW` when the tail block is full
3. writes through `GenericXLog` full-image page updates

It writes only the new node tuple. It does **not** patch existing nodes
or metadata.

New unit test:

- **IN-010** proves duplicate probing still finds a matching node that
  was appended after the grouped-codebook tail

### `routine.rs`

`ec_diskann_aminsert` now uses the 11036 forward-neighbor plan to do a
real non-empty append:

1. duplicate probe still runs first and still stops at the duplicate
   boundary
2. unique inserts still reuse the scan-side grouped-PQ candidate
   discovery + exact heap rerank path
3. the planned forward neighbors are now passed to
   `insert::append_live_node(...)`
4. the callback returns successfully instead of stopping at the old
   append/backlink boundary panic

Updated pg tests:

- **`test_ec_diskann_unique_insert_appends_live_node`** proves the
  second distinct insert now succeeds and leaves two heap rows in the
  table
- **`test_ec_diskann_duplicate_after_append_hits_boundary`** proves a
  later duplicate of that second vector is found through the appended
  node and still stops at the duplicate-bind boundary

The original duplicate-boundary test for the bootstrap node remains in
place.

## Boundary after this packet

Non-empty unique insert now:

- derives persisted payload
- rejects true duplicates
- finds exact forward neighbors
- appends the new node tuple with those forward neighbors

Non-empty unique insert still does **not**:

- write backlinks into existing nodes
- retry when a backlink target has changed
- update `inserted_since_rebuild`
- update `needs_medoid_refresh`
- grow overflow-heaptid chains
- make the appended node graph-reachable from the entry point

That last point is intentional for this slice: persisted duplicate
visibility is the goal here, not full graph connectivity.

## Tests

New coverage:

- **RD-020** grouped-codebook tuples are skipped by live-node iteration
- **RD-021** entry-point fallback can find a later appended node past a
  codebook gap
- **IN-010** duplicate probe sees a node appended after codebooks
- **`test_ec_diskann_unique_insert_appends_live_node`**
- **`test_ec_diskann_duplicate_after_append_hits_boundary`**

Retained coverage:

- **`test_ec_diskann_duplicate_insert_hits_duplicate_boundary`**
- **`test_ec_diskann_empty_index_bootstrap_insert_executes`**
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
- `cargo clippy --lib --no-deps` — clean apart from the known baseline
  `unnecessary_sort_by` warnings in `reader.rs`, `scan.rs`, and
  `vamana.rs`
- `cargo test --lib ec_diskann` — passed with `127 passed`, `0 failed`
- `cargo test` — passed
- `bash scripts/run_pgrx_pg17_test.sh` — passed
  The script is not executable in this checkout, so it was invoked via
  `bash`.
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  — still fails only on the known baseline warnings in untouched code:
  - existing lib warnings in `reader.rs`, `scan.rs`, and `vamana.rs`
  - existing test-only warnings in `scan.rs` and `vacuum.rs`

Note: an initial attempt to run `cargo test` and the pg17 script in
parallel collided on the shared `pgrx` test cluster (`postmaster.pid`).
The final verification results above are from serial reruns.

## Reviewer notes

- **Physical append is now real.** This packet writes a durable new node
  tuple for non-empty unique inserts.
- **Reader assumptions changed.** Reviewers should focus on the new
  mixed-tag traversal invariant: node iteration is tag-filtered, not
  tail-bounded.
- **No backlink or metadata writes yet.** Distinct appended nodes are
  persisted and duplicate-visible, but not yet fully integrated into
  the live Vamana graph.

## Not doing in this packet

- **Backlink writes**
- **Backlink replanning / stale-target retry**
- **Overflow-heaptid chain growth**
- **`inserted_since_rebuild` / medoid-refresh metadata updates**
- **Reachability guarantee for newly appended non-empty inserts**
