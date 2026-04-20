# Review Request: Empty-Index Bootstrap Insert (Phase 7B)

Branch: `adr034-diskann-rebased`
Author: coder-2
Target: `src/am/ec_diskann/routine.rs`,
`src/am/ec_diskann/insert.rs`,
`src/am/ec_diskann/ambuild.rs`

## What this packet is

This is the first actual `ec_diskann` `aminsert` landing, but only for
the ADR-046 step-9 special case:

- **empty index**
- **first live row**
- **metadata-serialized bootstrap**

Non-empty live insert is still explicitly rejected after bootstrap.

Concretely, this packet replaces the unconditional `aminsert` panic
with:

1. metadata-share read
2. empty-index detection
3. metadata-exclusive retry for the bootstrap case
4. one-row grouped-PQ training + one-node Vamana build
5. data-page append under the metadata bootstrap window
6. metadata publish with `inserted_since_rebuild = 1`

If the index is already non-empty, the callback now errors with the
explicit boundary message:

`ec_diskann non-empty aminsert is not yet implemented (task 17 phase 7)`

## Why this slice

ADR-046 explicitly carves out first-insert bootstrap as the one Phase 7
case that stays serialized under the metadata page. That makes it a good
vertical slice:

- it exercises real `aminsert` wiring
- it avoids backlink ordering and stale-target retry
- it avoids the still-unpinned overflow-chain anchor question
- it proves an empty built index can transition into a usable live
  index without going through `ambuild`

This packet builds on 11033's payload-derivation seam but does not yet
attempt the general non-empty graph-mutation path.

## What changed

### `routine.rs`

`ec_diskann_aminsert` now:

1. validates the single indexed datum is a non-null `ecvector`
2. decodes the heap TID into `storage::page::ItemPointer`
3. reads metadata under `BUFFER_LOCK_SHARE`
4. if the index is empty (`dimensions == 0 && entry_point == INVALID`),
   retries under `BUFFER_LOCK_EXCLUSIVE`
5. inside that bootstrap retry:
   - rechecks emptiness to tolerate a concurrent first insert
   - builds a one-row `EmptyInsertBootstrapOutput`
   - writes staged data pages with GenericXLog
   - replaces metadata in the same locked closure
6. returns `false` for the successful bootstrap insert
7. for any non-empty index, emits the explicit boundary error above

### `insert.rs`

The earlier Phase 7A payload-derivation module now also owns the
bootstrap helpers:

- **`read_metadata_page`** — block-0 share-lock decode
- **`with_locked_metadata_page`** — block-0 exclusive-lock mutate +
  GenericXLog full-image rewrite
- **`bootstrap_empty_insert_output`** — one-row grouped-PQ training,
  one-node Vamana build, codebook staging, metadata patch

`bootstrap_empty_insert_output` mirrors the build-side derivation rules:

- `seed = DEFAULT_QUANT_SEED`
- grouped-PQ group size via `ambuild::default_group_size`
- persisted binary sidecar derived only when the quantizer shape says it
  exists
- codebook chain staged and patched into `grouped_codebook_head`
- `inserted_since_rebuild = 1` for the first live row

### `ambuild.rs`

Three helpers widened to `pub(super)` so the insert bootstrap can reuse
the exact build-side logic instead of duplicating it:

- `default_group_size`
- `write_data_pages`
- `decode_heap_tid`

## Tests

Two new pg_tests in `routine.rs`:

- **`test_ec_diskann_empty_index_bootstrap_insert_executes`**
  - create empty table
  - create `ec_diskann` index
  - insert the first row
  - force planner away from seqscan / bitmapscan / sort
  - confirm `ORDER BY ... LIMIT 1` returns that row through the index

- **`test_ec_diskann_second_insert_still_errors`**
  - same setup
  - first insert bootstraps successfully
  - second insert is caught in a PL/pgSQL block and must fail with the
    explicit non-empty boundary message

The Phase 7A unit tests in `insert.rs` remain in place and still pass.

## Verification

```text
cargo build --lib
cargo clippy --lib --no-deps
cargo test --lib ec_diskann
cargo test
cargo pgrx test pg17
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Observed:

- `cargo build --lib` — passed
- `cargo clippy --lib --no-deps` — clean apart from the 8 pre-existing
  `unnecessary_sort_by` warnings in `reader.rs`, `scan.rs`, and
  `vamana.rs`
- `cargo test --lib ec_diskann` — `118 passed`, `0 failed`
- `cargo test` — passed
- `cargo pgrx test pg17` — passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  — still fails on pre-existing baseline warnings in untouched files
  (`reader.rs`, `scan.rs`, `vamana.rs`, plus existing test-only warnings
  in `scan.rs` and `vacuum.rs`); this packet introduces no new clippy
  findings on that gate

## Reviewer notes

- **This is ADR-046 step 9 only.** No non-empty insert path lands here.
  No backlink ordering, stale-target retry, duplicate binding, or
  overflow growth.
- **Metadata lock overlap is intentional in this slice.** The bootstrap
  case mirrors `ec_hnsw`'s first-insert special case: keep the empty
  shape transition serialized under the metadata page rather than trying
  to prematurely factor the general ordered-page-pass machinery into the
  first insert.
- **V0 still has no cold rerank writes.** Bootstrap writes only the hot
  node pages + grouped codebook chain + metadata. `rerank_tid` stays
  `INVALID`.
- **Why stop after the first row?** The next unresolved design seam is
  non-empty mutation, where duplicate binding, overflow growth, and
  backlink repair all share the ADR-046 ordered-page-pass rule. Keeping
  this packet to bootstrap-only avoids smuggling in a partial graph
  mutation story before that seam is explicit.

## Not doing in this packet

- **General non-empty `aminsert`**
- **Duplicate detection / duplicate bind**
- **Overflow-heaptid chain growth**
- **Backlink install / α-prune under page lock**
- **`MAX_BACKLINK_REPLAN_PASSES` retry loop**
- **Vacuum callback work**
