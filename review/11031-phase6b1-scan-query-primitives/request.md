# Review Request: Scan-Time Query + Codebook Primitives (Phase 6B-1)

Branch: `adr034-diskann-rebased`
Author: coder-2
Target: `src/am/ec_diskann/scan_query.rs` (new),
`src/am/ec_diskann/mod.rs`

## What this packet is

Pure-Rust Phase 6B-1 slice. Three primitives the pgrx scan callbacks
(Phase 6B-2) will consume:

1. **`read_grouped_codebook_chain(chain, head_tid, group_count,
   centroid_count) -> Vec<f32>`** — walks the persisted
   `VamanaCodebookTuple` chain from its head TID, concatenates shards'
   centroid payloads into the flat `[group][centroid * group_size]`
   layout that `build_grouped_pq_lut_f32` consumes. Verifies shards
   arrive in `group_index` order, the walk is exactly `group_count`
   long, and the chain terminates with `INVALID`.
2. **`encode_query_srht(raw_query, dimensions, seed) -> Vec<f32>`** —
   thin wrapper around `SrhtForwardTransform::for_dimensions(...)
   .apply(...)` so scan-time rotation agrees with ambuild's. Seed +
   dimensions come from the metadata page.
3. **`build_grouped_pq_lut_from_persisted(...)`** — convenience that
   reads the codebook chain, SRHT-encodes the query, and calls
   `build_grouped_pq_lut_f32` in one hop. Returns `(lut,
   group_count)` — the two arguments `grouped_pq_score_f32` needs.

## Why this

The V0 scan path has three moving pieces — persisted codebook, SRHT
rotation, grouped-PQ4 LUT — and all three must agree bit-for-bit with
ambuild's code derivation for scores to be meaningful. Extracting
them as pure-Rust primitives with their own tests means Phase 6B-2's
pgrx callbacks only have to thread arguments; the scoring loop is
already validated.

The end-to-end test (**CR-011**) is the seam check: it trains a
grouped-PQ4 model on synthetic vectors, stages the codebook through
Phase 5C-3's `stage_grouped_codebook_chain`, then rebuilds the LUT
from the persisted chain + a query vector via
`build_grouped_pq_lut_from_persisted`. The output must be
byte-identical to a directly-computed LUT over the in-memory model
— if it isn't, build-time and scan-time rotations have drifted.

## Tests

Seven new unit tests (nine counting error-path variants), all in
`scan_query.rs`:

- **CR-001** — multi-group codebook round-trips with shards in
  `group_index` order.
- **CR-002** — single-group codebook reads back cleanly.
- **CR-003** — `INVALID` head TID rejected before any page read.
- **CR-004** — declared group count too high (chain terminates early)
  errors with `"terminated early"`.
- **CR-005** — declared group count too low (chain runs past declared
  length) errors with `"longer than declared"`.
- **CR-006** — mismatched `centroid_count` propagates the length
  error from `VamanaCodebookTuple::decode`.
- **CR-010** — `encode_query_srht` is deterministic and output length
  equals `effective_transform_dim(dimensions)`.
- **CR-011** — end-to-end: persisted-path LUT matches in-memory LUT
  byte-for-byte on a trained model.
- **CR-012** — `group_size = 0` rejected before any chain read.

## Verification

```
cargo test --lib ec_diskann::scan_query     # 9 passed
cargo test --lib ec_diskann                 # 109 passed (was 100)
cargo clippy --lib --no-deps                # clean (8 pre-existing sort_by warnings in vamana.rs)
```

## Non-changes (affirming choices)

- **No shard caching.** Each `read_grouped_codebook_chain` call walks
  from the head. Phase 6B-2's pgrx scan calls this once at
  `amrescan`, not per `amgettuple`, so caching at this layer would be
  premature.
- **No reshape/transpose on the flat codebook.** The flat layout
  feeds `build_grouped_pq_lut_f32` directly; reshaping here would
  just cost a copy.
- **No prefilter-closure factory.** A convenience like `|tuple|
  grouped_pq_score_f32(&lut, group_count, &tuple.search_code)` is a
  one-line closure the Phase 6B-2 caller writes inline against the
  opaque. Wrapping it here adds an indirection without saving code.
- **Centroids retained as `f32`.** ADR-045 mandates fixed-length
  shards; compressing the codebook is out of scope for V0.

## Dependencies

- **Packet 11029** (VamanaCodebookTuple + codebook chain staging) —
  this is the reverse direction.
- **Packets 11017 / 11018** (persist sequencer, build orchestrator) —
  the shards this slice walks are what Phase 5C-3 wrote.
- **`plan/design/diskann-scan-pgrx.md`** — names "build prefilter
  LUT from quantizer scorer" as the Phase 6B seam; this slice
  materialises the LUT side.

## Not doing in this packet

- **pgrx scan callbacks.** Phase 6B-2 slice wires
  `ambeginscan`/`amrescan`/`amgettuple`/`amendscan` and replaces the
  "not yet implemented" stubs in `routine.rs`.
- **Chain materialization from a pg_sys relation.** The scan-callback
  slice will add a `materialize_chain_from_index(relation)` helper
  that walks `ReadBufferExtended` over blocks 1..n and builds a
  `DataPageChain`. Kept separate because it's the only pgrx-coupled
  piece of Phase 6B; everything else stays pure-Rust.
- **Heap rerank.** The exact-IP rerank closure (fetch ecvector from
  heap via `primary_heaptid`, compute `max(0, -ip)`) is Phase 6B-2's
  concern once the AM is actually being invoked.
- **SIMD LUT scoring.** `grouped_pq_score_f32` already exists; a
  SIMD-packed variant is a follow-on quantizer task, not a scan-side
  concern.
