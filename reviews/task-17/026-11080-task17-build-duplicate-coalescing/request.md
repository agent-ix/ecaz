# Review Request: coalesce DiskANN build duplicates into overflow chains

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `src/am/ec_diskann/ambuild.rs`
- `src/am/ec_diskann/insert.rs`
- `src/am/ec_diskann/routine.rs`

## What this packet is

This is the next DiskANN AM slice after packet `11079`.

While shaping the SQL-limit regression, a stronger bug surfaced: `CREATE INDEX
 ... USING ec_diskann` on a table that already held 12 identical rows only
 surfaced 5 rows through the built index. Live insert already handled duplicate
 vectors by binding them to one node plus overflow heap TIDs, but ambuild still
 persisted one graph node per scanned row and never staged an overflow chain.

This packet makes the build path match the live insert posture:

- exact duplicate source vectors share one DiskANN graph node
- extra heap TIDs stage into overflow tuples
- the built index returns every duplicate row through ordered SQL

## What changed

### `src/am/ec_diskann/ambuild.rs`

`RawHeapTuple` now tracks:

```rust
struct RawHeapTuple {
    primary_heap_tid: ItemPointer,
    overflow_heap_tids: Vec<ItemPointer>,
    source_vector: Vec<f32>,
}
```

and `BuildState::push(...)` now coalesces exact duplicate source vectors using
bit-exact float comparison:

```rust
if let Some(existing) = self
    .heap_tuples
    .iter_mut()
    .find(|existing| source_vectors_match_exactly(&existing.source_vector, &source_vector))
{
    existing.overflow_heap_tids.push(heap_tid);
    return;
}
```

The build graph / training input stays on the unique-vector set, and after
`build_and_persist_vamana(...)` the build path now stages overflow heap TIDs
onto the persisted owner node with:

```rust
insert::stage_overflow_heap_tids_in_chain(...)
```

before writing the chain to disk.

### `src/am/ec_diskann/insert.rs`

Added a pure chain helper:

```rust
pub(super) fn stage_overflow_heap_tids_in_chain(...)
```

It:

- decodes the owner node tuple from the in-memory `DataPageChain`
- rejects invalid / already-overflow / duplicate-primary inputs
- appends one or more `VamanaOverflowTuple`s
- patches `nexttid` links across multi-tuple overflow chains
- rewrites the owner node with `has_overflow_heaptids = true`

The new pure test
`in_005b_stage_overflow_heap_tids_in_chain_roundtrips_multiple_chunks`
proves the helper round-trips 12 overflow heap TIDs across multiple overflow
tuples and that `bound_heap_tids_for_owner(...)` decodes the same list back.

### `src/am/ec_diskann/routine.rs`

Added pg18 regression
`test_ec_diskann_build_coalesces_duplicate_vectors`:

1. create a heap table with 12 identical `ecvector` rows
2. build a default `ec_diskann` index over the populated table
3. assert the built chain has exactly one node tuple
4. assert that node advertises overflow heap TIDs and decodes 12 bound rows
5. force ordered index execution and prove `LIMIT 12` returns all 12 ids

That is the exact failing build posture this packet fixes.

## Why this slice

- DiskANN-only and directly on the AM behavior, not tooling.
- Fixes a real correctness hole in `ambuild`, not just a benchmark artifact.
- Aligns prebuilt indexes with the already-landed live insert duplicate path.
- Keeps the implementation local to DiskANN build / chain staging / pg tests.

## Test evidence

```text
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 218 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also passed on `pg18` for this checkpoint:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Notable new coverage in that run:

- `am::ec_diskann::insert::tests::in_005b_stage_overflow_heap_tids_in_chain_roundtrips_multiple_chunks`
- `am::ec_diskann::routine::tests::pg_test_ec_diskann_build_coalesces_duplicate_vectors`

## Follow-ups intentionally not in this packet

- Weighted training for duplicate-heavy corpora. This slice coalesces exact
  duplicates to preserve correctness and match live-insert semantics; it does
  not add sample weighting back into grouped-PQ training.
- Any change to non-identical near-duplicate handling. This packet is only
  about exact duplicate source vectors.
- Any extension of duplicate staging outside DiskANN. HNSW already has its own
  build-time duplicate coalescing path.
