# Task 87 Phase 1 CandidateBatch Design

## Decision

Introduce `CandidateBatch` as a shared, traversal-neutral scan scratch under
`src/am/common/`.

The abstraction is a borrowed candidate view, not an owner of index payloads:

```rust
pub struct CandidateBatch<'a, Id, Meta = CandidateMeta<'a>> {
    ids: Vec<Id>,
    payloads: Vec<CandidatePayload<'a, Meta>>,
    capacity: usize,
}

pub struct CandidatePayload<'a, Meta = CandidateMeta<'a>> {
    pub code: &'a [u8],
    pub meta: Meta,
}

pub enum CandidateMeta<'a> {
    None,
    Gamma(f32),
    GammaAndResidualSigns { gamma: f32, signs: &'a [u8] },
    Binary,
    RaBitQ,
    GroupedPq { group_count: usize },
}
```

The exact Rust names can change during implementation, but the contract should
not:

- AMs push `(id, code borrow, metadata)` in traversal order.
- `id` is AM-owned and can be an `ItemPointer`, row offset, `SpireVecId`, or a
  small struct containing the materialization fields needed after scoring.
- `code` is borrowed from an already-live page, object column, or decoded tuple.
  `CandidateBatch` never fabricates a long-lived reference from a raw pointer.
- metadata is an enum or typed sidecar that describes per-vector scoring data
  without naming a quantizer family in the batch type.
- scoring consumers fill a caller-owned score buffer in candidate order.
- AMs own heap/top-k/frontier behavior after the score buffer comes back.

This keeps the batch in the AM traversal layer while leaving quantizer kernels
under `src/quant/`. The first kernel consumer is TurboQuant no-QJL 4-bit:

```rust
score_tq_no_qjl_4bit_batch(prepared, batch, scores)
```

The initial implementation may use the current per-candidate LUT scorer inside
that batch entry point. A 32-vector u8 nibble LUT kernel should only land after
the per-AM batch integration proves that the AMs can feed useful batch sizes.

## Lifetime And Safety Contract

`CandidateBatch` must remain safe Rust.

- It may store `&'a [u8]` payload views whose lifetime is tied to the decoded
  tuple, page object, or column block borrowed by the caller.
- It must not expose safe APIs that convert `*mut T` or page offsets into
  arbitrary `&'a T`.
- If an AM needs to read from PostgreSQL pages, that AM continues to own the
  unsafe page access and passes only validated slice views into the batch.
- If a future SIMD kernel needs `unsafe`, that boundary stays inside
  `src/quant/` and gets its own safety docs and scalar differential coverage.

## Batch Size Policy

Use a small default capacity in the shared type, with AM-local overrides:

- default: 64 candidates;
- SPIRE: leaf-column chunks, usually `min(row_count, 256)` because column
  payloads are already contiguous;
- IVF: posting-list chunks, `64` or `128` candidates, with scalar tail handled
  by the same batch flush path;
- DiskANN: per-expanded-node neighbor set or per-page decode chunk, whichever
  is available after tuple reads; expected batch size is `R`/page bounded;
- HNSW: per-successor expansion batch, capped by layer neighbor slots (`m` or
  `2m`) because greedy search depends on each step's scored frontier.

The batch owns only vectors of ids and borrowed payload views. It does not copy
payload bytes unless a later AM proves its source storage cannot keep borrows
alive through scoring.

## Quantizer-Agnostic Walkthrough

TurboQuant no-QJL 4-bit:

- metadata: `CandidateMeta::None` or `Gamma(0.0)`;
- code: MSE packed code bytes;
- score path: in-scope Task 87 batch scorer.

TurboQuant no-QJL 2-bit:

- metadata: none;
- code: MSE packed code bytes;
- batch scorer can reuse the same `code: &[u8]` and prepared query shape with a
  different kernel.

TurboQuant QJL:

- metadata: `Gamma(f32)` if QJL signs are packed into `code`, or
  `GammaAndResidualSigns` if a future storage format splits hot/cold side data;
- the batch type does not assume gamma is absent.

RaBitQ:

- metadata: none for current IVF/DiskANN estimator paths;
- code: RaBitQ code bytes;
- later batch scorer can call the existing `PreparedEstimator::estimate_ip_batch`
  shape without changing `CandidateBatch`.

Binary fingerprint:

- metadata: none;
- code: binary words exposed as bytes or through a typed binary candidate view;
- Hamming kernels can consume the same candidate-id/score-buffer order.

PQ / grouped-PQ:

- metadata: group shape if not captured entirely by the prepared scorer;
- code: grouped codes;
- IVF/DiskANN existing grouped-PQ LUT scoring maps directly to a batch scorer.

This walkthrough finds no TurboQuant-only assumption in the batch contract.

## Per-AM Mapping

SPIRE:

- Current scoring site: `src/am/ec_spire/scan/candidates.rs`, V2 column path
  calls `SpirePreparedAssignmentScorer::score_batch_ip` over contiguous column
  payloads.
- Phase 2 replaces the local SPIRE batch loop with the shared `CandidateBatch`
  scorer for TurboQuant no-QJL 4-bit only.
- RaBitQ cutoff and selected-block pruning paths stay on their current scalar
  or RaBitQ-specific paths until follow-up kernels exist.

IVF:

- Current scoring surface: `src/am/ec_ivf/quantizer.rs::score_ip_from_parts`.
- Phase 3 batches posting-list rows before heap/top-k insertion. The batch id
  should include posting row location and heap TID so scoring remains separate
  from candidate materialization.
- Existing RaBitQ `score_ip_bits1_batch_from_payloads` proves a batch-shaped
  helper already fits IVF, but Task 87 routes only TQ no-QJL 4-bit through the
  new shared abstraction.

DiskANN:

- Current scan prefilter surface: `src/am/ec_diskann/quantizer.rs` and
  `src/am/ec_diskann/scan.rs`, with grouped-PQ and RaBitQ search codecs.
- The current checkout does not expose a DiskANN TurboQuant search codec. Phase
  4 must first confirm whether a TurboQuant no-QJL DiskANN search codec exists
  on the target branch/history or whether Task 87's DiskANN slice should define
  a Stop Condition or a narrow codec-enablement prerequisite.
- If TurboQuant is available, the batch should flush per expanded neighbor set
  after tuple decode, preserving Vamana's best-first control flow.

HNSW:

- Current scoring site: `src/am/ec_hnsw/scan.rs::score_tq_distance` and graph
  successor callbacks in `src/am/ec_hnsw/graph.rs`.
- Phase 5 batches neighbors within one successor expansion. It must not batch
  across greedy-search steps because the next expanded node depends on scored
  results from the current step.
- The batch id can be `ItemPointer`; HNSW already materializes graph elements
  before invoking score callbacks.

## pgvectorscale Resort Buffer Comparison

pgvectorscale's `TSVResponseIterator` owns a `resort_buffer` with capacity
`TSV_RESORT_SIZE` and fills it by repeatedly calling `next()`, fetching full
distances for returned candidates, and popping the best resorted item.

That is the right production pattern to copy at the contract level:

- collect a bounded group of traversal-produced candidates;
- score/rescore them outside the graph traversal primitive;
- emit in sorted order without changing storage layout.

It differs from Task 87 in two important ways:

- pgvectorscale's resort buffer is a result-iteration buffer, while
  `CandidateBatch` is a scoring input buffer used before AM-specific top-k or
  frontier updates;
- Task 87's batch preserves AM-owned ids and borrowed compressed code slices,
  not heap/index pointers plus full-distance scores.

## Streaming ANN Composition

Task 88 should be able to place a result/resort iterator above the same scoring
surface:

- traversal produces candidate ids and borrowed payload views;
- `CandidateBatch` scores a bounded batch;
- the AM or streaming iterator decides whether to insert into frontier, top-k,
  or resort buffer.

The batch must not own cursor state, snapshot state, heap slots, or PostgreSQL
scan descriptors. That keeps streaming iteration free to hold and refill its
own resort buffer without rewriting the scoring API.

## Kernel Decision

Do not land a new 32-vector u8 nibble LUT kernel in Phase 1.

The first implementation should land the abstraction and route each AM through
a batch scorer that preserves exact scores. Once all four AMs expose real batch
sizes and scoring-share counters, add the 32-vector kernel only where measured
batch sizes justify it. This avoids freezing a flat-slab layout before HNSW and
DiskANN prove their traversal-local batch shapes.

## Measurement Methodology

Each AM slice uses `ecaz bench suite` with a checked-in suite JSON under that
packet. The suite shape follows Task 86 packet 008:

- corpora: DBPedia real10k, real50k, real100k;
- storage format: TurboQuant no-QJL 4-bit where supported;
- comparison: baseline source install from pre-slice commit vs change source
  install from slice commit;
- output: recall@10, latency p50/p95/p99, storage, scoring-share counters;
- isolation: one index per table, recorded explicitly in `manifest.md`;
- recall gate: byte-equal top-k/truth output for every cell;
- storage gate: unchanged index/table bytes except expected metadata-free
  noise from rebuilds.

The suite runner must preserve `suite.json`, `suite-manifest.json`,
`results.jsonl`, raw recall/latency/storage logs, and report output under the
owning review packet's `artifacts/`.

## Review Questions

1. Is the borrowed payload + typed metadata contract sufficiently
   quantizer-agnostic for the follow-up quantizer table?
2. Should DiskANN's missing TurboQuant search codec be handled as a prerequisite
   slice, a Stop Condition, or evidence that the task wording needs adjustment?
3. Is deferring the 32-vector u8 nibble LUT until after per-AM batch integration
   the right risk tradeoff?
