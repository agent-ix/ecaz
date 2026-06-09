# Task 91 Phase 1 Trait Audit

Head SHA: `5cdcf38a529ddec50665a4ea44b806f03383897f`

This is the design-only Phase 1 audit for Task 91. It records the current
per-AM scoring surfaces and the migration contract before any Rust code
changes.

## Current Common Surface

`src/am/common/quant_codec.rs` currently defines exactly one `QuantCodec`
trait:

- `codec_kind() -> QuantCodecKind`
- `search_codec_tag() -> QuantSearchCodecTag`
- `payload_len() -> usize`
- `encode_source(&[f32]) -> EncodedQuantPayload`
- `prepare_ip_query(&[f32]) -> PreparedQuery`
- `score_ip_candidate(prepared, CandidatePayload) -> f32`
- `score_ip_batch(prepared, CandidateBatch, out_scores)`

`src/am/common/candidate_batch.rs` already carries the per-candidate metadata
needed by known quant families:

- `None`
- `Gamma(f32)`
- `GammaAndResidualSigns { gamma, signs }`
- `Binary`
- `RaBitQ`
- `GroupedPq { group_count }`

The existing metadata enum is sufficient for the Phase 2-6 migration. The
important trait growth is not a new metadata variant; it is explicit model and
dispatch ownership.

## Batch Method Name

Keep `QuantCodec::score_ip_batch` as the universal block-kernel dispatch entry
point for Task 91 and Task 92.

Rationale:

- all live compressed-domain AM scan scoring is inner-product scoring;
- Task 87 packets and current code already use `score_ip_batch`;
- renaming to `score_batch` would create churn before a non-IP quantized scan
  metric exists;
- future non-IP work can add a metric-specific sibling or a metric enum once a
  real call site needs it.

Task 92 should register scalar/SIMD block kernels under this method.

## Dispatch Shape

Use enum dispatch at the AM boundary, not `dyn QuantCodec` in hot scoring
loops.

`QuantCodec` has an associated `PreparedQuery` type, so trait objects would
need a type-erased prepared-query wrapper. That would move dynamic checks into
the hottest scan code. Current AMs already have concrete enums for prepared
query state. Phase 2-6 should keep the concrete enum shape, but make the enum
variants delegate through the shared `QuantCodec` contract.

The recommended shape is:

```text
AM storage binding enum
  -> selected QuantCodec enum/concrete adapter
     -> concrete PreparedQuery enum
        -> score_ip_batch
```

`DispatchedQuantCodec` may be introduced if two or more AMs need the same
runtime container. It should remain an enum over concrete adapters, not a boxed
trait object.

## Kernel Registration Contract

Each `QuantCodec` implementation owns the whole scoring decision:

1. Validate batch/output lengths and payload shapes.
2. If `batch.len() >= 32`, call the quant kernel module for whole block32
   ranges.
3. Score any tail through the scalar reference path.
4. If `batch.len() < 32`, use scalar only.
5. Attribute counters by `(am, quant_kind, isa)` after a successful score.

AM scan loops should not call `src/quant/<kernel>/{neon,sve,avx2}` directly.
They pass borrowed candidate payloads through `CandidateBatch` and receive
scores in candidate order.

Scalar implementations must be bit-exact with the pre-migration scorer. SIMD
variants may use the ADR-076 tolerance: at most 4 ULP or `1e-6` relative error,
with recall@k preservation as the bench-level gate.

## Grouped-PQ Model Ownership

The current `encode_source`/`prepare_ip_query` surface is not enough for
grouped-PQ where trained codebooks are persisted AM state. IVF already exposes
`prepare_ip_query_with_pq_model`, and DiskANN reads grouped codebooks from the
data page chain during prefilter preparation.

Do not force trained model ownership into `CandidateBatch` metadata. The
model is query/codec state, not per-candidate metadata.

Phase 2 should grow the common surface with a model-aware construction path.
Preferred shape:

```text
QuantCodec implementation owns or borrows its trained model before
prepare_ip_query().
```

In concrete terms, IVF's PqFastScan adapter should become the reference:

- resolve storage/profile shape normally;
- bind the `IvfPqFastScanModel` into the codec adapter before query prep;
- `PreparedQuery::PqFastScan` owns the LUT and suffix-max state;
- candidate metadata remains `CandidateMeta::GroupedPq { group_count }`.

DiskANN and HNSW should follow the same division: page/metadata readers load
model bytes, the QuantCodec adapter owns scoring state, and `CandidateBatch`
only carries borrowed candidate code plus small per-candidate metadata.

## Residual Signs

`CandidateMeta::GammaAndResidualSigns` is the right place for QJL
residual-sign sidecars. Phase 4 HNSW migration should use it for
TurboQuant exact paths that need both gamma and signs.

The no-QJL 4-bit lanes should continue to use `CandidateMeta::Gamma` or
`CandidateMeta::None`; the scorer ignores gamma where the current math ignores
it.

## AM Path Audit

| AM | Current scoring surface | QuantCodec fit | Migration disposition |
|---|---|---|---|
| IVF TurboQuant generic | `IvfPreparedQuery::TurboQuant` and `score_ip_from_parts` | Already partially implemented by `impl QuantCodec for IvfQuantizer` | Phase 2 retouches after model/dispatch growth. |
| IVF TurboQuant no-QJL 4-bit | `IvfPreparedQuery::TurboQuantNoQjl4BitLut` plus Task 87 LUT32 batch scorer | Fits `score_ip_batch` directly | Keep as reference for width gating and counters. |
| IVF RaBitQ | `IvfPreparedQuery::RaBitQ`, scalar and existing payload batch helpers | Fits `score_ip_batch`; batch kernel comes later | Move bound-aware helpers under codec-adapter API without changing scan pruning. |
| IVF grouped-PQ/PqFastScan | `IvfPreparedQuery::PqFastScan` with `IvfPqFastScanModel` loaded by scan/build code | Fits after model binding growth | Phase 2 makes this the model-ownership reference. |
| SPIRE assignment TurboQuant | `SpirePreparedAssignmentScorer::TurboQuant`; `SpireAssignmentQuantCodec` exists but duplicate scorer methods remain | Fits today for assignment payloads | Phase 3 routes selected-block and remaining assignment batch calls through `QuantCodec`. |
| SPIRE assignment RaBitQ | `SpirePreparedAssignmentScorer::RaBitQ`; scalar path inside scorer | Fits after prepared scorer delegates through trait | Phase 3 preserves cutoff/bound logic and replaces inline scoring. |
| SPIRE PqFastScan | currently rejected without persisted grouped-PQ model | Same model-binding issue as IVF | Phase 3 only routes live paths; PqFastScan remains model-gated. |
| HNSW TurboQuant exact | `TurboQuantExactScoreMode::{Exact,FullLut,TiledLut,Int8Approx}` in scan state | Fits as TurboQuant codec variants / prepared query variants | Phase 4 collapses scorer math into QuantCodec; traversal mode stays HNSW-owned. |
| HNSW gamma fallback | direct `ProdQuantizer::score_ip_from_parts` | Fits with `CandidateMeta::Gamma` or `GammaAndResidualSigns` | Phase 4 routes via codec scalar path and preserves score polarity. |
| HNSW PqFastScan | graph storage descriptors plus PqFastScan traversal/rerank decisions | Fits after model binding, but traversal/rerank policy stays HNSW-owned | Phase 4 moves grouped scoring math only. |
| HNSW RaBitQ | `RaBitQScorer` stored in scan opaque | Fits as RaBitQ QuantCodec prepared query | Phase 4 moves score call through codec and preserves HNSW negative-distance polarity. |
| DiskANN binary sidecar | `DiskannPreparedPrefilter::BinarySidecar` Hamming scorer | Fits as `QuantCodecKind::Binary` | Phase 5 adds Binary codec adapter; storage sidecar stays DiskANN-owned. |
| DiskANN grouped-PQ | `DiskannPreparedPrefilter::GroupedPq` plus codebooks from chain | Fits after model binding | Phase 5 moves score math through GroupedPq codec. |
| DiskANN RaBitQ | `DiskannPreparedPrefilter::RaBitQ` scalar estimator | Fits as RaBitQ codec | Phase 5 moves score math through codec. |
| DiskANN TurboQuant | missing search codec discriminator and prefilter scorer | Fits as TurboQuant codec | Phase 6 lands Task 90 content through QuantCodec. |

## Adapter Names

Rename storage-binding types where "codec" currently means AM layout rather
than quant scorer:

- `HnswStorageCodec` -> `HnswStorageBinding`
- `DiskannBuildCodec` -> `DiskannBuildBinding` or `DiskannStorageBinding`

`DiskannPreparedPrefilter` may keep its current name during Phase 5 if it only
represents scan policy, but its quant scoring arms must delegate into
`QuantCodec`.

The invariant after Task 91 closeout: "codec" without a storage qualifier means
the shared quant scoring contract, not AM tuple/page layout.

## Phase 2 Reference Slice

Phase 2 should update IVF first because it already implements the trait and
covers TurboQuant, RaBitQ, and grouped-PQ in one module.

Recommended Phase 2 deliverables:

- preserve `score_ip_batch` name;
- add model-bound grouped-PQ codec construction for IVF;
- move IVF RaBitQ/TurboQuant specialized batch helpers behind the trait method;
- align counters with Task 92 if Task 92 Phase 2 has landed; otherwise keep
  Task 87 counter behavior unchanged;
- add focused unit tests proving grouped-PQ model binding through the trait.

No AM storage format changes are part of Phase 2.
