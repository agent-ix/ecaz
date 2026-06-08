# Task 87 Packet 001: Phase 1 CandidateBatch Design

## Summary

This is the required Phase 1 design packet before any AM-level refactor.

The proposed shape is a shared `CandidateBatch` under `src/am/common/`: a safe
borrowed candidate-view abstraction holding AM-owned ids plus borrowed code
slices and typed per-candidate metadata. Quantizer-specific kernels consume the
batch and fill a score buffer; AMs keep traversal, top-k, frontier, and heap
materialization ownership.

## Artifact

- `artifacts/candidate-batch-design.md`
- `artifacts/manifest.md`

## Design Calls

- The batch contract is quantizer-agnostic. It can host TurboQuant no-QJL,
  TurboQuant QJL, RaBitQ, binary fingerprint, and PQ/grouped-PQ metadata shapes.
- Phase 2-5 should route TurboQuant no-QJL 4-bit through the abstraction in
  SPIRE, IVF, DiskANN, and HNSW, but keep other quant paths on their current
  scorers until follow-up kernels land.
- The new 32-vector u8 nibble LUT kernel should not land in Phase 1. First prove
  that all four AMs can feed useful batch sizes through the shared contract.
- Task 88 streaming should compose by putting its resort/result buffer above
  the same score-buffer contract; `CandidateBatch` must not own cursor or scan
  descriptor state.

## DiskANN Scope Note

The current checkout's DiskANN search codecs are grouped-PQ and RaBitQ. I did
not find a DiskANN TurboQuant search codec to route through the Task 87 TQ
no-QJL 4-bit path. Review should explicitly decide whether Phase 4 needs a
small prerequisite codec-enablement slice, a Stop Condition, or a wording
adjustment before implementation proceeds.

## Validation

No tests were run. This packet is documentation-only and contains no code or
behavioral change.

## Review Focus

- Confirm the lifetime and metadata contract does not bake in TurboQuant-only
  assumptions.
- Confirm the per-AM mapping is compatible with existing SPIRE, IVF, DiskANN,
  and HNSW scoring sites.
- Confirm whether deferring the 32-vector kernel until after per-AM batch
  plumbing is acceptable.
- Resolve the DiskANN TurboQuant scope question before Phase 4.
