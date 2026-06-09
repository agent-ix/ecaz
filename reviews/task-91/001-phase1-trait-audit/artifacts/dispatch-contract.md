# Task 91 Phase 1 Dispatch Contract

Head SHA: `5cdcf38a529ddec50665a4ea44b806f03383897f`

This addendum makes the Phase 2 implementation contract concrete enough for
review. It is intentionally written as an implementation target, not as final
Rust code.

## Trait Surface

Keep the existing trait name and batch method:

```rust
pub(crate) trait QuantCodec {
    type PreparedQuery;

    fn codec_kind(&self) -> QuantCodecKind;
    fn search_codec_tag(&self) -> QuantSearchCodecTag;
    fn payload_len(&self) -> usize;
    fn encode_source(&self, source: &[f32]) -> Result<EncodedQuantPayload, String>;
    fn prepare_ip_query(&self, query: &[f32]) -> Result<Self::PreparedQuery, String>;
    fn score_ip_candidate(
        &self,
        prepared_query: &Self::PreparedQuery,
        payload: CandidatePayload<'_>,
    ) -> Result<f32, String>;
    fn score_ip_batch<Id>(
        &self,
        prepared_query: &Self::PreparedQuery,
        batch: &CandidateBatch<'_, Id>,
        out_scores: &mut [f32],
    ) -> Result<(), String>;
}
```

Phase 2 should not rename `score_ip_batch`. Task 92 can use that method as its
kernel registration entry point.

## Model-Bound Construction

Grouped-PQ needs trained codebooks before query preparation. Do not add model
bytes to `CandidateMeta`.

Add a small adapter-level construction pattern instead:

```text
IvfQuantizerProfile + optional IvfPqFastScanModel
  -> IvfQuantCodec
     -> IvfPreparedQuery
```

The concrete name can vary, but the behavior must be:

- TurboQuant and RaBitQ adapters are constructible from the storage/profile
  metadata alone.
- Grouped-PQ adapters are constructible only when the AM has supplied the
  persisted grouped-PQ model.
- `prepare_ip_query` fails early if a grouped-PQ adapter is missing its model.
- `score_ip_batch` never reads model bytes from AM storage; it only consumes
  prepared-query state and candidate code bytes.

This is the reference pattern DiskANN and HNSW should follow later with their
own metadata/page readers.

## Prepared Query Enum

Keep prepared-query enums concrete:

```text
IvfPreparedQuery::{TurboQuant, TurboQuantNoQjl4BitLut, RaBitQ, PqFastScan}
SpirePreparedAssignmentScorer::{TurboQuant, RaBitQ, ...}
HnswPreparedQuantQuery::{TurboQuantExact, TurboQuantFullLut, ...}
DiskannPreparedQuantPrefilter::{Binary, GroupedPq, RaBitQ, TurboQuant}
```

The migration target is not "delete every enum"; it is "make enum variants
delegate through the one `QuantCodec` contract and stop carrying standalone
quant kernel knowledge in AM scan loops."

## AM-Owned Versus Quant-Owned State

AM-owned and kept local:

- HNSW traversal policy, frontier order, score polarity, tuple ownership, and
  `TurboQuantExactScoreMode` selection.
- DiskANN graph reader, page-chain reads, sidecar presence checks, and prefilter
  policy.
- IVF posting-list scratch layout, bound-pruning call sites, and list-level
  scan state.
- SPIRE selected-block routing, placement policy, and object/leaf read policy.

Quant-owned and moved under `QuantCodec`:

- source-vector to quant search-code encoding;
- query LUT/estimator/prepared scorer construction once model bytes are bound;
- code-byte shape validation;
- scalar scoring math;
- batch scoring math and Task 92 kernel dispatch.

## Candidate Metadata Rules

Use metadata variants narrowly:

- `None`: quant family needs only code bytes.
- `Gamma(f32)`: TurboQuant generic scalar/QJL paths that need gamma.
- `GammaAndResidualSigns { gamma, signs }`: QJL residual-sign sidecar paths.
- `Binary`: Hamming/binary sidecar paths.
- `RaBitQ`: RaBitQ payloads when AMs want explicit metadata validation.
- `GroupedPq { group_count }`: grouped-PQ payloads when the candidate-side
  group count is worth validating against prepared-query state.

Do not store trained codebooks, query LUTs, relation handles, or page-chain
state in candidate metadata.

## Width Gate

Every QuantCodec batch scorer uses the same outer rule:

```text
if batch.len() >= 32:
  score whole block32 range through Task 92 kernel module
  score tail through scalar reference
else:
  score all candidates through scalar reference
```

Task 91 Phase 2 may keep the current Task 87 LUT32 implementation as-is until
Task 92 backfills the module layout, but any new batch path must use this rule.

Scalar tails after one or more ISA kernel blocks are attributed to `isa=Scalar`
in Task 92 counters. Do not record tails under the selected ISA row's
`scalar_*` fields. This keeps Task 93-98 counter rows comparable across hosts
and across dispatch outcomes.

## Error and Counter Ordering

Implementation order inside `score_ip_batch`:

1. Validate output length equals batch length.
2. Validate prepared-query variant matches codec kind.
3. Validate all candidate payload lengths and metadata kinds needed by the
   selected scorer.
4. Score candidates.
5. Increment counters only after successful scoring.

This preserves the current behavior where failed shape validation does not
produce misleading benchmark counters.

## Phase 2 Acceptance Evidence

The IVF retouch packet should include:

- focused unit tests for TurboQuant, no-QJL LUT32, RaBitQ, and grouped-PQ
  through `QuantCodec::score_ip_batch`;
- one `f32::to_bits()` parity test per prepared-query shape:
  `TurboQuant`, `TurboQuantNoQjl4BitLut`, `RaBitQ`, and `PqFastScan`;
- one grouped-PQ test proving model-bound construction is required and works;
- one mismatch test proving grouped-PQ without a model fails before scoring;
- `git diff --check` artifact;
- no storage-format changes.
