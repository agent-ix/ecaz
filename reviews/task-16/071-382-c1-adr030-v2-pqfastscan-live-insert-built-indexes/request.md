# Review Request: C1 ADR-030 V2 PqFastScan Live Insert On Built Indexes

## Context

Packet 381 moved exact graph read/search behind `GraphStorageDescriptor`, so
insert traversal and backlink scoring no longer assumed scalar tuple layout.

That still left live insert itself blocked for `PqFastScan`:

- duplicate detection still errored before any grouped insert could proceed
- duplicate coalescing only knew how to rewrite scalar element tuples
- append still only knew how to write scalar neighbor + element tuple pairs

At the same time, the grouped build path already had most of the payload pieces:

- persisted grouped codebooks
- grouped search-code derivation from source vectors
- hot/cold payload staging via `stage_v2_grouped_build_payload(...)`

So the next useful slice was to support live insert into a non-empty built
`PqFastScan` index while leaving empty grouped-index initialization explicitly
out of scope.

## Problem

Before this packet, every `PqFastScan` live insert failed with:

- `tqhnsw aminsert does not support PqFastScan indexes yet`

That was too coarse after packet 381:

1. a built grouped index already has persisted codebooks and enough runtime
   metadata to derive grouped search codes
2. grouped duplicate coalescing only needs to update the grouped hot tuple's
   inline heap-TID list
3. the real remaining blocker is empty-index bootstrap, where there are no
   persisted grouped codebooks yet

So "all grouped insert is unsupported" was no longer the right architecture
boundary.

## Planned Slice

One live-insert checkpoint:

1. allow live insert into a non-empty built `PqFastScan` index
2. derive grouped search codes from persisted grouped codebooks at insert time
3. append grouped neighbor + rerank + hot tuples on the live path
4. support duplicate coalescing against existing grouped hot tuples
5. keep empty grouped-index first insert explicitly unsupported

Vacuum parity remains out of scope for this packet.

## Implementation

Updated:

- `src/am/insert.rs`
- `src/lib.rs`

### 1. Grouped duplicate scan no longer hard-rejects

`InsertFormatAdapter::find_duplicate(...)` now:

- keeps the scalar duplicate scan for `TurboQuant`
- scans grouped hot tuples for `PqFastScan`
- loads the cold rerank tuple to compare exact `gamma + code`

So duplicate matching now works against built grouped indexes instead of
throwing the old top-level unsupported error.

### 2. Grouped duplicate coalescing rewrites the hot tuple in place

Live duplicate coalescing is now adapter-owned:

- `TurboQuant` still rewrites `TqElementTuple`
- `PqFastScan` rewrites `TqGroupedHotTuple`

The grouped path only mutates the inline heap-TID list. Rerank and neighbor
payloads remain untouched, which matches the grouped build layout.

### 3. Built grouped indexes can now append live hot/cold payloads

`src/am/insert.rs` now adds a real grouped append path:

- derive grouped search code from the persisted grouped codebook chain
- stage hot/cold payload through `build::stage_v2_grouped_build_payload(...)`
- append:
  - neighbor tuple
  - rerank tuple
  - grouped hot tuple

Like the scalar path, this still prefers tail-page reuse and falls back to a
fresh page when the current tail page lacks enough free space.

### 4. Empty grouped-index first insert now errors for the real reason

The old blanket grouped insert error is replaced with a narrower boundary:

- `tqhnsw aminsert requires a prebuilt PqFastScan index with persisted grouped codebooks`

This is surfaced when the index is still effectively empty from the grouped
runtime perspective:

- no grouped codebook head
- zero grouped search shape in metadata

That makes the remaining limitation explicit instead of pretending all grouped
live insert is unsupported.

### 5. Added pg coverage for the new grouped insert contract

`src/lib.rs` now covers:

- empty grouped-index insert rejection with the new codebook-specific error
- successful live insert into a built grouped index
- grouped duplicate coalescing on the live insert path

The success coverage also checks that live grouped insert keeps writing grouped
hot/rerank/neighbor tuples instead of falling back to scalar element tuples.

## Measurements

No new benchmark or recall measurements in this slice. This is live-insert
functional parity groundwork only.

## Validation

Passed:

- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands still fail on this workstation at the same known
PostgreSQL linker layer as prior checkpoints:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed failure mode is unchanged:

- unresolved PostgreSQL symbols during link, including
  `CurrentMemoryContext`, `PG_exception_stack`, `error_context_stack`, and
  `errstart`

## Outcome

This checkpoint changes grouped live insert from a blanket reject into a more
accurate split:

1. built `PqFastScan` indexes can now accept live inserts
2. grouped duplicate coalescing works on the live path
3. grouped live append writes hot + cold payloads in the runtime layout
4. empty grouped-index bootstrap is still explicitly unsupported

What it still does **not** do:

- first insert into an empty `PqFastScan` index
- `PqFastScan` vacuum cleanup / finalize
- grouped insert-time caching of grouped codebooks or derived search-code state

## Next Slice

The next practical slices are:

1. remove the top-level `PqFastScan` vacuum reject by teaching vacuum how to
   finalize and repair grouped hot/cold tuples
2. decide whether empty `PqFastScan` index bootstrap should remain disallowed
   or gain a separate codebook-training/bootstrap path
