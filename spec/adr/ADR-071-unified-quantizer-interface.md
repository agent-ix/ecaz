---
id: ADR-071
title: "Unified Quantizer Interface Across Access Methods"
status: PROPOSED
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

Adopt a unified quantizer interface as the target architecture, but do not
start a flag-day extraction as part of the DiskANN RaBitQ integration.

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

This is intentionally illustrative, not a committed API. The final API should
come from the second and third AM integrations, after repeated call sites are
clear.

## Current Implementation Guidance

Task 60 may use a DiskANN-local adapter that delegates to shared quantizer
kernels, because that keeps the RaBitQ integration small and preserves DiskANN
scan invariants:

- relation-graph reads stay on-demand;
- RaBitQ metadata gets an explicit DiskANN discriminator;
- `pq_fastscan` on-disk metadata remains backward compatible;
- no AM is forced to accept a cross-AM trait before HNSW and IVF requirements
  are visible together.

The adapter should not fork the quantizer math. It may own only AM binding
logic: metadata mapping, payload sizing, payload encoding, prepared scorer
selection, and score polarity normalization.

## Extraction Trigger

Start the shared interface extraction only when at least two of these are true:

1. HNSW RaBitQ integration duplicates the DiskANN/IVF build or scan adapter
   shape.
2. IVF needs to consume the same prepared-query scorer or metadata surface as
   DiskANN/HNSW.
3. A new quantizer family would otherwise require parallel build/scan plumbing
   in two or more AMs.
4. Benchmark/reporting rows need a single quantizer identity object to avoid
   drift between AM-specific reloption labels and on-disk discriminators.

At extraction time, the first PR should move only the stable common contract and
one family implementation behind it. It should not rewrite all AMs in one
commit. DiskANN and HNSW can adopt the shared interface before IVF if their
graph payload surfaces converge first.

## Consequences

- **Short-term velocity stays high.** DiskANN RaBitQ can land with a narrow
  adapter instead of a broad quantizer abstraction.
- **The direction is explicit.** Future HNSW and IVF work should avoid inventing
  unrelated quantizer APIs where the same concepts are appearing.
- **On-disk compatibility remains AM-owned.** The shared quantizer interface
  does not own graph/page tuple formats; each AM still owns its format
  discriminator and upgrade rules.
- **Extraction has a measurable trigger.** We wait for duplicated shape, not an
  abstract desire for tidiness.

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
