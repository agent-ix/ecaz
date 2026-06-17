---
type: ADR
id: ADR-071
title: "Unified Quantizer Interface Across Access Methods"
status: ACCEPTED
impact: Affects ADR-031, ADR-041, ADR-048, FR-013, FR-034, FR-035, FR-038, and future HNSW/DiskANN/IVF storage-format work.
date: 2026-05-25
---
# ADR-071: Unified Quantizer Interface Across Access Methods

## Context

Ecaz now has multiple access methods that need the same quantizer families in
different physical layouts:

- `ec_ivf` already supports RaBitQ as a posting-list storage/profile.
- Task 60 adds DiskANN `storage_format = 'rabitq'` alongside the existing
  DiskANN `pq_fastscan` payload.
- Follow-on work is expected to add RaBitQ to HNSW and then keep IVF, HNSW, and
  DiskANN behavior aligned.

The current code has useful shared quantizer kernels under `src/quant/`, but
the access-method integration surfaces differ:

- IVF owns posting-list storage and list-probe scoring.
- HNSW owns graph tuple payloads, build/link scoring, and ordered scan state.
- DiskANN owns Vamana tuple payloads, metadata discriminators, relation-graph
  reads, and on-demand scan prefilters.

That means a quantizer family is not yet a drop-in component. Each AM still
decides how to train, encode, persist, prepare the query-side scorer, score
candidates, expose reloptions, and validate on-disk metadata.

## Decision

Adopt `QuantCodec` (`src/am/common/quant_codec.rs`) as the accepted shared
quantizer-family scoring interface across access methods.

This accepts the Task 91 extraction for the quantizer-family layer only. Access
methods still own storage binding, tuple/list/page layout, traversal rules,
relation access, and on-disk compatibility. ADR-072 records that boundary.

The common scoring contract is:

- `encode_source` prepares family-owned payload code bytes;
- `prepare_ip_query` builds the family-owned query scorer state;
- `score_ip_candidate` scores one encoded candidate;
- `try_score_ip_candidate` optionally applies a caller-provided inner-product
  cutoff while preserving the same candidate scoring semantics;
- `score_ip_batch` is the universal batch entry point used by AM scan loops
  and by Task 92 block-kernel dispatch infrastructure;
- `codec_kind`, `search_codec_tag`, and `payload_len` provide stable
  quantizer-family identity for reporting and validation.

The original no-flag-day guidance remains correct for storage adapters and
on-disk layout. Task 91 deliberately extracted the stable quantizer-family
contract through reviewed slices instead of rewriting every AM in one commit.

The long-term interface should separate three concerns:

1. **Quantizer family semantics.** Training, deterministic parameterization,
   encode/decode where applicable, prepared-query construction, score polarity,
   and persisted family metadata.
2. **AM payload binding.** How an AM embeds the quantized bytes and metadata
   into graph tuples, posting-list tuples, sidecars, or segment records.
3. **AM traversal contract.** How the AM consumes approximate scores while
   preserving its own traversal and IO rules.

In Rust terms, the eventual shape should be closer to:

```rust
pub trait QuantizerFamily {
    type BuildState;
    type PreparedQuery;

    fn train(&self, vectors: &[&[f32]], dim: usize) -> Self::BuildState;
    fn code_len(&self, build: &Self::BuildState, dim: usize) -> usize;
    fn encode(&self, build: &Self::BuildState, vector: &[f32], out: &mut [u8]);
    fn prepare_query(&self, build: &Self::BuildState, query: &[f32]) -> Self::PreparedQuery;
    fn score(&self, prepared: &Self::PreparedQuery, code: &[u8]) -> f32;
    fn metadata(&self, build: &Self::BuildState) -> QuantizerMetadata;
}

pub trait QuantizedPayloadCodec {
    fn payload_code_len(&self) -> usize;
    fn write_code(&self, code: &[u8], dst: &mut Vec<u8>);
    fn read_code<'a>(&self, payload: &'a [u8]) -> &'a [u8];
}
```

This sketch remains useful context for the separation of concerns, but the
accepted Rust API is `QuantCodec`. Future metric-specific or non-IP scan paths
should add explicit siblings instead of weakening the current IP scoring
contract.

## Task 91 Adoption Record

Task 91 migrated the quantizer-family scoring layer onto `QuantCodec` across
the active AM surfaces:

- IVF uses `IvfQuantizer` / `IvfQuantCodec`, including model-bound grouped-PQ,
  RaBitQ, TurboQuant, batch scoring, and cutoff scoring.
- HNSW routes TurboQuant exact modes, grouped-PQ, and RaBitQ approximate scan
  scoring through HNSW-local `QuantCodec` adapters.
- DiskANN routes binary-sidecar, grouped-PQ, RaBitQ, and TurboQuant prefilter
  scoring through DiskANN-local `QuantCodec` adapters.
- SPIRE routes selected-row, sampled-row, routed-probe, column-batch, and
  bounded RaBitQ cutoff scoring through `SpireAssignmentQuantCodec`.

The accepted contract includes `try_score_ip_candidate`; it retired the final
SPIRE bounded-cutoff exception and is part of the common surface available to
future early-pruning work.

## Current Implementation Guidance

DiskANN, HNSW, IVF, and SPIRE may use AM-local adapters that implement
`QuantCodec` or return a bound `QuantCodec` view. That keeps storage and
traversal invariants local while preventing quantizer math and scoring
semantics from forking. For DiskANN, this preserves the original Task 60
invariants:

- relation-graph reads stay on-demand;
- RaBitQ metadata gets an explicit DiskANN discriminator;
- `pq_fastscan` on-disk metadata remains backward compatible;
- no AM is forced to accept a cross-AM trait before HNSW and IVF requirements
  are visible together.

The adapter should not fork the quantizer math. It may own only AM binding
logic: metadata mapping, payload sizing, payload encoding, prepared scorer
selection, and score polarity normalization.

## Extraction Trigger

The quantizer-family extraction trigger has fired and is satisfied by Task 91.
Do not reopen the broad question for IP scan scoring without new contradictory
evidence.

Further extraction is still gated for storage-binding traits. Promote
index-local codec adapters into a shared storage trait only when at least two
of these are true:

1. HNSW RaBitQ integration duplicates the DiskANN/IVF build or scan adapter
   shape.
2. IVF needs to consume the same prepared-query scorer or metadata surface as
   DiskANN/HNSW.
3. A new quantizer family would otherwise require parallel build/scan plumbing
   in two or more AMs.
4. Benchmark/reporting rows need a single quantizer identity object to avoid
   drift between AM-specific reloption labels and on-disk discriminators.

At storage-adapter extraction time, the first PR should move only the stable
common storage contract and one family implementation behind it. It should not
rewrite all AMs in one commit.

## Consequences

- **Quantizer scoring is shared.** New IP scoring paths should enter through
  `QuantCodec`, including batch and cutoff-capable scoring.
- **Short-term storage velocity stays high.** AMs can keep narrow storage
  bindings instead of forcing a broad storage abstraction.
- **The direction is explicit.** Future HNSW, DiskANN, SPIRE, and IVF work
  should avoid inventing unrelated quantizer scoring APIs where `QuantCodec`
  already holds the common contract.
- **On-disk compatibility remains AM-owned.** The shared quantizer interface
  does not own graph/page tuple formats; each AM still owns its format
  discriminator and upgrade rules.
- **Storage extraction still has a measurable trigger.** We wait for duplicated
  storage-binding shape, not an abstract desire for tidiness.

## Alternatives Considered

### Extract the shared interface during DiskANN RaBitQ

Rejected for now. DiskANN has stricter scan-path constraints than the interface
can express today, and HNSW RaBitQ has not yet shown its final payload binding
shape. Extracting now risks freezing DiskANN-specific assumptions into a
nominally shared trait.

### Keep every AM-specific quantizer adapter permanently

Rejected as the strategic direction. It would make each new quantizer family pay
the same integration cost three times and would increase the chance that
`storage_format = 'rabitq'` means subtly different things across AMs.

### Put on-disk payload encoding inside the quantizer family

Rejected. Quantizer families own code bytes and scoring semantics, but AMs own
tuple/page layout, relation reads, WAL posture, and metadata page contracts.
Conflating those layers would make a shared quantizer harder, not easier.

## Related Decisions

- ADR-031: RaBitQ binary prefilter, superseded by the landed first-class RaBitQ
  quantizer.
- ADR-041: module structure for multi-AM, multi-quantizer growth.
- ADR-048: IVF access method and its existing RaBitQ storage-format surface.
- ADR-070: on-disk forward-compat encoding convention.
