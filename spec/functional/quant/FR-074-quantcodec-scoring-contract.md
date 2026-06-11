---
id: FR-074
title: "QuantCodec Scoring Contract"
artifact_type: FR
status: IMPLEMENTED
relationships:
  - target: "ix://agent-ix/ecaz/US-002"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-014"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/ecaz/FR-015"
    type: "references"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-063"
    type: "publishes"
    cardinality: "1:1"
---
# [FR-074] QuantCodec Scoring Contract

## Description

`QuantCodec` is the shared compressed-domain scoring interface selected by
ADR-071/ADR-072 and required by `FR-014`: every AM scan loop routes
compressed-domain scoring through an index-local adapter implementing this
contract, and AM code never calls ISA-specific kernel functions directly.
This FR pins the contract's method surface, which until now was named
throughout the spec but defined only in code.

No spec-objects kind models a code interface/trait; this FR is authored as a
standard behavioral FR (recorded as a format gap).

Implementation anchor: `src/am/common/quant_codec.rs`.

## Specification

### Inputs

| Field | Type | Source |
| --- | --- | --- |
| `source` | `&[f32]` | Raw vector for `encode_source`. |
| `query` | `&[f32]` | Raw query for `prepare_ip_query`. |
| `payload` | `CandidatePayload<'_>` | One candidate's persisted code bytes plus format side data (gamma, sidecar words) as stored by the owning AM. |
| `batch` | `&CandidateBatch<'_, Id>` | Borrowed candidate batch assembled by the AM scan loop. |
| `min_ip_to_keep` | `Option<f32>` | Optional cutoff for `try_score_ip_candidate`. |

### Outputs

| Field | Type | Description |
| --- | --- | --- |
| `EncodedQuantPayload` | struct | Persisted-format code bytes from `encode_source`. |
| `Self::PreparedQuery` | associated type | Codec-owned prepared-query state (LUTs, projected query, scales). |
| `f32` / `Option<f32>` | score | Inner-product estimate per candidate; `None` when cut off. |
| `out_scores: &mut [f32]` | batch scores | Candidate-order scores from `score_ip_batch`. |
| counter rows | `FR-063` snapshot | `(surface, quant_kind, isa)` attribution emitted by batch dispatch. |

### Behavior

The trait surface SHALL be:

```rust
pub(crate) trait QuantCodec {
    type PreparedQuery;
    fn codec_kind(&self) -> QuantCodecKind;
    fn search_codec_tag(&self) -> QuantSearchCodecTag;
    fn payload_len(&self) -> usize;
    fn encode_source(&self, source: &[f32]) -> Result<EncodedQuantPayload, String>;
    fn prepare_ip_query(&self, query: &[f32]) -> Result<Self::PreparedQuery, String>;
    fn score_ip_candidate(&self, prepared_query: &Self::PreparedQuery,
        payload: CandidatePayload<'_>) -> Result<f32, String>;
    fn try_score_ip_candidate(&self, prepared_query: &Self::PreparedQuery,
        payload: CandidatePayload<'_>, min_ip_to_keep: Option<f32>)
        -> Result<Option<f32>, String>;
    fn score_ip_batch<Id>(&self, prepared_query: &Self::PreparedQuery,
        batch: &CandidateBatch<'_, Id>, out_scores: &mut [f32])
        -> Result<(), String>;
}
```

1. `codec_kind` SHALL return one of the seven closed `QuantCodecKind` values
   whose labels are pinned by `FR-063` (`turboquant`, `turboquant_qjl`,
   `turboquant_tiled_lut`, `turboquant_int8`, `rabitq`, `grouped_pq`,
   `binary`).
2. `payload_len` SHALL be constant for a codec instance so AMs can validate
   stored payload bytes before scoring.
3. `score_ip_batch` SHALL own shape prevalidation, width-cascade routing
   (`FR-014`), runtime ISA dispatch, scalar-remainder fallback, and counter
   attribution. Output scores SHALL be in candidate order. Prevalidation
   failures SHALL mutate no output slot and no counter.
4. `score_ip_candidate` SHALL remain the scalar correctness anchor: batch and
   per-candidate routes satisfy the family's ADR-076 anchor mode against the
   same scalar reference.
5. Index-local adapters (for example `HnswTurboQuantScanCodec`,
   `DiskannTurboQuantPrefilterCodec`) SHALL bind AM-owned storage metadata
   and traversal state to the codec, per `FR-015`'s adapter rules; quant code
   owns scoring math and dispatch, AM code owns storage and traversal.
6. New quant families SHALL enter through this contract — scalar reference,
   batch dispatch, width cascade, counters, reporting — rather than adding
   AM-specific scoring entry points.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-074-CON-1 | AM scan code does not call ISA-specific kernel functions directly; the codec method is the only compressed-domain scoring entry | Architecture | Code review (ADR-076) |
| FR-074-CON-2 | `QuantCodecKind` is a closed enum shared with the counter surface; adding a kind updates `FR-063` and `NFR-015` in the same change | Architecture | Spec review |
| FR-074-CON-3 | Batch scoring with a cutoff never reorders or skips output slots; cutoffs are per-candidate (`try_score_ip_candidate`) | Technical | Unit test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-074-AC-1 | Every shipped quant family scores through a `QuantCodec` implementation on every batchable AM surface, observable via `FR-063` counter rows | pg_test + counter audit |
| FR-074-AC-2 | Batch and per-candidate routes agree within the family's ADR-076 anchor mode on shared fixtures | Unit test |
| FR-074-AC-3 | A shape-invalid batch returns an error with no score output mutation and no counter increment | Unit test |

## Dependencies

- **Upstream**: ADR-071/ADR-072 (interface selection), ADR-076 (kernel pattern), `FR-015` (ProdQuantizer math and adapter rules).
- **Downstream**: `FR-014` accelerated surfaces, `FR-063` counter snapshot, `FR-067`/`FR-068` scan pipelines, Task 99 completeness matrix.
