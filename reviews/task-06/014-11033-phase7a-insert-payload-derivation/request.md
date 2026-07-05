# Review Request: Insert Payload Derivation from Persisted Codebooks (Phase 7A)

Branch: `adr034-diskann-rebased`
Author: coder-2
Target: `src/am/ec_diskann/insert.rs` (new),
`src/am/ec_diskann/mod.rs`

## What this packet is

This is the first narrow Phase 7 prep slice for `ec_diskann`.

Before the pgrx `aminsert` callback can mutate any graph pages, it
needs one pure-Rust seam: given the built index's metadata page and
persisted grouped-PQ codebook chain, derive the new node's persisted
payload from an incoming raw `ecvector` source row.

This packet adds exactly that seam:

- **`derive_insert_payload_from_persisted(metadata, chain, source)`**
  in `src/am/ec_diskann/insert.rs`
- **`DerivedInsertPayload { binary_words, search_code }`**

The pgrx callback itself still panics in `routine.rs`. No page writes,
duplicate binding, backlink repair, or metadata mutation land here.

## Why this slice exists

Phase 7 still has two separable concerns:

1. turn a heap-row source vector into the exact bytes a new Vamana node
   would persist
2. decide where and how to write that node under ADR-046's lock ordering

The first concern is independent of the second and already fully
specified by the built index's metadata:

- SRHT seed and dimensions live on block 0
- grouped-PQ search shape (`search_subvector_count`, `_dim`) lives on
  block 0
- the persisted grouped codebook chain hangs off
  `metadata.grouped_codebook_head`
- the persisted binary sidecar bit lives in `payload_flags`

Locking this seam now means later insert slices can focus on graph
mutation rather than re-deriving the payload contract in the middle of
page-local write logic.

## What changed

New module: `src/am/ec_diskann/insert.rs`

### Public-in-module API

```rust
pub(super) struct DerivedInsertPayload {
    pub(super) binary_words: Vec<u64>,
    pub(super) search_code: Vec<u8>,
}

pub(super) fn derive_insert_payload_from_persisted(
    metadata: &VamanaMetadataPage,
    chain: &DataPageChain,
    source_vector: &[f32],
) -> Result<DerivedInsertPayload, String>
```

### Derivation rules

The helper:

1. validates metadata shape up front:
   - non-zero `dimensions`
   - source dimension matches metadata
   - `transform_kind == SRHT`
   - `search_codec_kind == GROUPED_PQ`
   - non-zero grouped search shape
   - `grouped_codebook_head != INVALID`
2. loads the flat grouped codebooks from the persisted codebook chain
   via `read_grouped_codebook_chain`
3. SRHT-rotates the raw source vector with `encode_query_srht`
4. encodes grouped-PQ nibbles with `encode_grouped_pq`
5. derives persisted binary sidecar words only when
   `PAYLOAD_FLAG_BINARY_SIDECAR` is set, reusing the same
   `ProdQuantizer::cached(...)` +
   `training::derive_persisted_binary_words(...)` contract as build

This gives Phase 7 the exact `search_code` / `binary_words` pair that a
new node tuple should carry, without touching neighbors or lock order.

## Tests

Five unit tests in `insert.rs`:

- **IN-001** — payload derivation matches the build-side grouped-PQ
  code and persisted binary sidecar words on a trained model
- **IN-002** — clearing `PAYLOAD_FLAG_BINARY_SIDECAR` yields an empty
  `binary_words` payload while preserving grouped-PQ search code
- **IN-003** — missing `grouped_codebook_head` errors
- **IN-004** — source / metadata dimension mismatch errors
- **IN-005** — unsupported transform kind / codec kind are rejected

## Verification

```text
cargo build --lib
cargo clippy --lib --no-deps
cargo test --lib ec_diskann
```

Observed:

- `cargo build --lib` — passed
- `cargo clippy --lib --no-deps` — clean apart from the 8 pre-existing
  `unnecessary_sort_by` warnings in `reader.rs`, `scan.rs`, and
  `vamana.rs`
- `cargo test --lib ec_diskann` — `116 passed`, `0 failed`

## Reviewer notes

- **No pgrx callback change in this slice.** `ec_diskann_aminsert`
  still errors in `routine.rs`; this packet only lands the payload
  derivation seam that later callback slices will call.
- **No cold rerank chain work.** Per ADR-046 / ADR-047 frozen rules,
  V0 insert derives only the hot payload bytes. `rerank_tid` remains
  `INVALID` and `PAYLOAD_FLAG_COLD_RERANK_PAYLOAD` stays out of scope.
- **Why not overflow-chain primitives first?** The current slim node
  layout exposes `has_overflow_heaptids` but does not yet materialize a
  chain anchor in code. Rather than inventing a wire contract inside a
  mutation slice, this packet locks the unambiguous payload seam first.

## Not doing in this packet

- **Duplicate detection / duplicate bind**
- **Overflow-heaptid chain growth**
- **Backlink planning or α-prune under write lock**
- **Metadata mutation (`inserted_since_rebuild`)**
- **Entry-point repair / `needs_medoid_refresh` handling**
- **pgrx `aminsert` callback wiring**
