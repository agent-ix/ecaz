# Review Request: Grouped-PQ4 Codebook Chain Staging (Slice A of Phase 5C-3)

Branch: `adr034-diskann-rebased`
Author: coder-2
Target: `src/am/ec_diskann/tuple.rs`, `src/am/ec_diskann/persist.rs`
Commit: `d89b917`

## What this packet is

Pure-Rust Slice A of Phase 5C-3. Two additions, no pgrx:

1. **`VamanaCodebookTuple`** (tag `0x07`) in `tuple.rs` — slim tuple
   that carries one grouped-PQ4 codebook shard: `group_index: u16`,
   `nexttid: ItemPointer` (chain link), `centroids: Vec<f32>`. Symmetric
   `encode` / `decode` / `encoded_len`, foreign-tag and
   wrong-length rejection.

2. **`stage_grouped_codebook_chain(chain, model) -> Result<ItemPointer, String>`**
   in `persist.rs` — appends each shard of a `GroupedPq4Model` to the
   shared `DataPageChain` using the same placeholder-then-patch
   discipline as `persist_vamana_graph`: every shard is first inserted
   with `ItemPointer::INVALID` as `nexttid`, then patched in reverse
   with the TID of the following shard. Returns the head TID for the
   caller to drop into `VamanaMetadataPage::grouped_codebook_head`.

## Why this

Grouped-PQ4 codebook shards share a storage contract with the graph
node tuples: they live on the same `DataPageChain`, they follow the
shared fixed-length-update discipline, and Phase 5D's reader can walk
them via `nexttid`. By putting both into the chain under a single
generic XLog transaction at build time, the scan path gets an atomic
metadata+graph+codebook image — there is no intermediate on-disk state
where metadata points at a partial codebook.

The placeholder-then-patch pattern is a hard requirement of ADR-045
(Page Layout Discipline, packet 11014): every persisted tuple must
keep its byte length invariant across updates, so `nexttid` must be
encoded at full width the first time the shard is written.

## Tests

- **LA-030..LA-033** (tuple.rs): round-trip; placeholder and patched
  forms are the same length; foreign tag rejected; wrong length
  rejected.
- **CB-001..CB-004** (persist.rs): single-shard chain; multi-shard
  chain TIDs link in order; empty model is a no-op returning
  `INVALID`; patched `nexttid` fields match the head → tail traversal.

## Verification

```
cargo check --lib                                  # clean
cargo test --lib ec_diskann::tuple                 # LA-030..LA-033 green
cargo test --lib ec_diskann::persist               # CB-001..CB-004 green
```

## Non-changes (affirming choices)

- `VamanaMetadataPage::grouped_codebook_head` stays `INVALID` at
  `empty()` time. The ambuild caller patches it after the chain is
  staged (Slice B). This keeps `empty()` meaningful for the
  ambuildempty path and for tests that don't need a codebook.
- No codebook caching on the reader. Codebooks are loaded lazily on
  scan open (Phase 6B), not pinned at AM init.
- The tag byte `0x07` is distinct from the graph-node tag to keep
  a mixed-chain walker's `match` exhaustive. Tag registry is tracked
  inline in `tuple.rs` top-of-file comment.

## Dependencies

- **Packet 11014** (ADR-045 page-layout discipline) — the fixed-length
  invariant this slice upholds.
- **Packet 11017** (Phase 5C-1 persist sequencer) — `DataPageChain`
  primitive that this slice extends with a second tuple type.
- **Packet 11018** (Phase 5C-2 build orchestrator) — defines the
  `GroupedPq4Model` shape whose shards this slice persists.

## Not doing in this packet

- Reading codebook shards back. Phase 5D's `PersistedGraphReader`
  learns about the codebook only when scan actually needs it
  (Phase 6B).
- Rebuild/migration of codebook shards after vacuum. ADR-047 §10
  defers codebook re-training to full rebuild.
- Per-group compression of the centroid payload. Codebook shards are
  small (≤ group-size × dim × 4 bytes) and the V0 format keeps them
  plain f32.
