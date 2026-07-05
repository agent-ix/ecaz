# Review Request: Exact Duplicate-Probe Boundary for Non-Empty Insert (Phase 7C)

Branch: `adr034-diskann-rebased`
Author: coder-2
Target: `src/am/ec_diskann/insert.rs`,
`src/am/ec_diskann/routine.rs`

## What this packet is

This is the first non-empty insert slice after 11034's empty-index
bootstrap landing.

It still does **not** mutate a non-empty `ec_diskann` graph. Instead, it
lands the duplicate-probe boundary that Phase 7 will need before any
duplicate bind or new-node append path can be made safe:

1. derive the incoming row's persisted payload from the built index's
   metadata + grouped codebooks
2. scan only duplicate-eligible live node tuples for payload matches
3. confirm those candidates with **exact heap `ecvector` equality**
4. emit a duplicate-specific boundary error only for true duplicates

Unique non-empty inserts still stop at the existing generic Phase 7
boundary.

## Why this slice

The first attempt at a non-empty duplicate probe used persisted payload
equality alone (`binary_words` + `search_code`). That turned out to be
too weak for the bootstrap case: with a one-row grouped-PQ model,
distinct source vectors can quantize to the same persisted payload.

So this packet freezes the correct split:

- **payload equality is only a coarse candidate filter**
- **duplicate identity is exact heap-vector equality**

That keeps the eventual duplicate-bind path honest without forcing any
overflow-chain or backlink writes into this slice.

## What changed

### `insert.rs`

Added:

- **`duplicate_candidate_tids_by_payload(reader, metadata, payload)`**

This helper:

1. walks persisted node TIDs in physical order
2. stops before the appended grouped-codebook chain
3. ignores tombstoned tuples
4. ignores stripped / primary-less tuples (`primary_heaptid == INVALID`)
5. returns the candidate node TIDs whose persisted payload matches the
   incoming derived payload

Important detail: it is intentionally a **candidate** probe, not a final
duplicate verdict.

New unit tests:

- **IN-006** returns both live matching node TIDs in physical order
- **IN-007** skips deleted and stripped tuples even when the persisted
  payload matches

### `routine.rs`

`ec_diskann_aminsert` now, on the non-empty path:

1. materializes the persisted chain
2. derives the incoming payload from persisted grouped codebooks
3. runs the payload candidate probe
4. for each candidate:
   - reopens the node tuple
   - fetches the candidate heap row via `SnapshotSelf`
   - detoasts the indexed `ecvector`
   - compares the raw `Vec<f32>` to the incoming source vector
5. raises:

`ec_diskann duplicate bind is not yet implemented (task 17 phase 7): existing node at (...)`

only when the heap vectors match exactly

If no exact duplicate is found, the callback still raises the generic
boundary:

`ec_diskann non-empty aminsert is not yet implemented (task 17 phase 7)`

New pg test:

- **`test_ec_diskann_duplicate_insert_hits_duplicate_boundary`**
  confirms an identical second row now hits the duplicate-specific
  boundary while 11034's existing
  `test_ec_diskann_second_insert_still_errors` continues to prove that a
  distinct second row hits the generic non-empty boundary.

## Why exact heap equality matters here

`ec_hnsw` can define duplicate equality from richer persisted payload
surfaces (`code` + `gamma`, or grouped/turbo rerank payload bytes). V0
`ec_diskann` does not have an index-owned cold rerank payload. The hot
payload (`binary_words` + grouped search code) is enough to find likely
candidates, but not enough to prove duplicate identity after a
bootstrap-trained one-row codebook.

So this packet intentionally uses:

- persisted payloads for candidate narrowing
- heap `ecvector` equality for the final duplicate decision

## Tests

New coverage:

- **IN-006** candidate probe returns matching live node TIDs
- **IN-007** candidate probe ignores deleted / stripped tuples
- **`test_ec_diskann_duplicate_insert_hits_duplicate_boundary`**
- existing **`test_ec_diskann_second_insert_still_errors`** remains
  meaningful and still passes with a distinct second vector

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
- `cargo test --lib ec_diskann` — `121 passed`, `0 failed`
- `cargo test` — passed
- `cargo pgrx test pg17` — passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  — still fails on pre-existing baseline warnings in untouched files
  (`reader.rs`, `scan.rs`, `vamana.rs`, plus existing test-only warnings
  in `scan.rs` and `vacuum.rs`); this packet introduces no new clippy
  findings on that gate

## Reviewer notes

- **No non-empty graph mutation yet.** This packet does not append a
  new node, bind a duplicate heap TID, grow an overflow chain, or touch
  backlinks.
- **Deleted / stripped tuples are ineligible duplicate targets.** This
  follows the ADR-047 reviewer guidance that Phase 7 duplicate lookup
  must ignore tombstoned tuples and tuples whose `primary_heaptid` has
  already been stripped.
- **Codebook chain is not part of duplicate scan.** The candidate probe
  stops at `grouped_codebook_head`, relying on the Phase 5C-3 / 6B-2
  dense append contract that grouped codebooks are staged after node
  tuples.
- **Exact comparison uses the indexed `ecvector` column.** No cold
  rerank payload is introduced; V0 still reranks and now duplicate-checks
  from the heap row.

## Not doing in this packet

- **Duplicate bind write path**
- **Overflow-heaptid chain growth**
- **True new-node append on non-empty indexes**
- **Backlink planning / ordered rewrite passes**
- **`inserted_since_rebuild` updates for non-empty inserts**
