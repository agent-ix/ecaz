# Task 87 Phase 1 Common Codec and Broad Batch Scope Addendum

This addendum resolves reviewer addenda
`reviews/task-87/001-phase1-design/feedback/2026-06-08-02-reviewer.md`
and
`reviews/task-87/001-phase1-design/feedback/2026-06-08-03-reviewer.md`.

It supersedes the narrower TQ-only wording in packets 002-006. Those
packets remain useful structural slices, but they no longer represent
the complete Task 87 scope.

## Scope Changes

Task 87 now has two shared surfaces:

1. `CandidateBatch`: the borrowed candidate data-flow view used by AM
   traversal code to flush a batch of candidate ids, score codes, and
   per-candidate metadata.
2. Common quant codec shape: the build/query/scoring/search-code tag
   surface used to register quantizers into AMs without each AM creating
   a bespoke enum.

Batching is architectural. The default is to route a quant mode through
`CandidateBatch` when the AM traversal exposes useful batch boundaries.
A "no" cell must cite structural or measured evidence.

## Common Quant Codec Shape

The shared shape should support these operations:

- `encode_source`: build/insert-side conversion from source vector to
  persisted search-code bytes plus per-row metadata.
- `prepare_query`: scan-side query preparation, including LUTs, rotated
  query buffers, estimators, and binary query words.
- `score_one`: legacy scalar scorer for paths whose batch size is one
  or whose routing table justifies scalar scoring.
- `score_batch`: scorer over `CandidateBatch<'_, Id, CandidateMeta>`,
  filling an output score buffer in candidate order.
- `candidate_meta`: typed per-candidate metadata such as gamma,
  residual signs, grouped-PQ shape, or no metadata.
- `search_codec_kind`: persisted or in-memory tag used by each AM to
  identify the search-code format.

The implementation can be a trait object, a shared enum over concrete
codecs, or an enum plus trait helpers. The required property is that a
new quantizer implementation plugs into AMs through one registration
surface instead of four unrelated AM-specific enum families.

## AM-Specific Enum Migration

Collapse into or adapt through the common shape:

- `IvfPreparedQuery::{TurboQuant, TurboQuantNoQjl4BitLut, RaBitQ,
  PqFastScan}`.
- `DiskannBuildCodec::{PqFastScan, RaBitQ}`.
- `DiskannPreparedPrefilter::{BinarySidecar, GroupedPq, RaBitQ}`.
- `SpirePreparedAssignmentScorer::{TurboQuant, RaBitQ}` and the
  TurboQuant `no_qjl_4bit_lut` optional prepared state.

May remain AM-specific:

- HNSW traversal policy and `TurboQuantExactScoreMode` as a kernel
  selection detail, as long as the underlying TurboQuant codec and
  batch scorer come from the common shape.
- AM scheduling, frontier, prefetch, dedup, and top-k/rerank ownership.

## Migration Order

1. Stabilize `CandidateBatch` and `CandidateMeta` as the shared
   candidate view.
2. Introduce the common quant codec trait/enum surface using the closest
   existing implementation as the first adapter. IVF is the best first
   target because `IvfPreparedQuery` already covers TurboQuant, RaBitQ,
   and grouped-PQ/PqFastScan in one enum.
3. Move SPIRE and HNSW structural routes to consume the common
   TurboQuant adapter instead of calling the TurboQuant helper directly.
4. Extend DiskANN build/prefilter codec registration to grouped-PQ,
   RaBitQ, binary-sidecar, and TurboQuant through the common shape.
5. Land real batch kernels per quant mode where measurements show enough
   batch width to justify them.

## Routing Table

| AM | Quant mode | Route through `CandidateBatch`? | Batch boundary | Notes |
|---|---|---:|---|---|
| SPIRE | TurboQuant no-QJL 4-bit | yes | assignment payload chunks | Packet 003 covers first structural route. |
| SPIRE | generic TurboQuant gamma-aware/QJL | yes, pending adapter | assignment payload chunks | Needs metadata for gamma/residual signs. |
| SPIRE | RaBitQ assignment | yes, pending adapter | assignment payload chunks | Prepared estimator carries query-side metadata today. |
| IVF | TurboQuant no-QJL 4-bit LUT | yes | posting-list scratch SoA flush | Packet 004 covers first structural route. |
| IVF | generic TurboQuant gamma-aware/QJL | yes, pending adapter | posting-list scratch SoA flush | Needs residual-sign metadata if QJL scorer is batched. |
| IVF | RaBitQ | yes | posting-list scratch SoA flush | Existing RaBitQ helper should be moved behind common batch scorer. |
| IVF | grouped-PQ/PqFastScan | yes | posting-list scratch SoA flush | Classic PQ LUT batch shape. |
| DiskANN | grouped-PQ prefilter | yes | expanded-node/page candidate set | Existing in-scope path; packet 005 Stop Condition is superseded. |
| DiskANN | RaBitQ prefilter | yes | expanded-node/page candidate set | Existing in-scope path. |
| DiskANN | binary-sidecar | yes | expanded-node/page candidate set | Hamming/Jaccard batch scorer expected to be worthwhile. |
| DiskANN | TurboQuant | yes, codec gap | expanded-node/page candidate set | Missing codec must be closed through common shape. |
| HNSW | TQ `FullLut` | yes | per-successor expansion batch | Packet 006 covers first structural route with owned payload backing. |
| HNSW | TQ `TiledLut` | yes, pending adapter | per-successor expansion batch | Tile-local reuse must be preserved. |
| HNSW | TQ `Int8Approx` | yes, pending adapter | per-successor expansion batch | Approximate scorer needs its own batch adapter. |
| HNSW | generic gamma-aware fallback | justify before routing | per-successor expansion batch | Batch sizes may be small; scalar may remain if measured/structural evidence says batch adds overhead. |
| HNSW | RaBitQ/grouped/binary if present | yes if batch-shaped | per-successor expansion batch | Borrow source follows the same owned-scratch rule. |
| Any AM | f32 raw | no by default | varies | Per-vector SIMD is already efficient; routing requires measured benefit. |

## HNSW Borrow Source

HNSW uses owned scratch or owned loaded payload bytes as the backing for
borrowed `CandidateBatch` entries. It must not hold transient tuple/page
borrows across a batch flush. Packet 006 implements this for newly
loaded `FullLut` exact payloads by preserving `LoadedElementState`
payload bytes until the flush.

## DiskANN Resolution

Packet 005 is now superseded as a final Stop Condition. It remains valid
source evidence that DiskANN lacks a TurboQuant codec today, but Task 87
must not use it to skip DiskANN. The DiskANN slice now includes:

- grouped-PQ prefilter batch routing;
- RaBitQ prefilter batch routing;
- binary-sidecar batch routing where applicable;
- TurboQuant codec registration through the common shape.

If the common codec surface itself proves too large for a single AM
slice, land that surface as a prerequisite Task 87 packet rather than
deferring DiskANN out of scope.

## Impact on Existing Packets

- Packet 003: SPIRE TQ no-QJL route remains useful but incomplete.
- Packet 004: IVF TQ no-QJL route remains useful but incomplete.
- Packet 005: DiskANN Stop Condition is superseded by this addendum.
- Packet 006: HNSW `FullLut` route remains useful but incomplete because
  `TiledLut`, `Int8Approx`, and any other batch-shaped HNSW quant paths
  still need routing decisions/adapters.
