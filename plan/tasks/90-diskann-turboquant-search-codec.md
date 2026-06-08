# Task 90: DiskANN TurboQuant Search Codec Prerequisite

Status: absorbed by Task 87 scope revision (2026-06-08)
Owner: coder (to be assigned). One coder, one branch.
Priority: 2 (Task 87 DiskANN prerequisite)

2026-06-08 update: Task 87 reviewer addenda
`reviews/task-87/001-phase1-design/feedback/2026-06-08-02-reviewer.md`
and
`reviews/task-87/001-phase1-design/feedback/2026-06-08-03-reviewer.md`
broadened Task 87. DiskANN's missing TurboQuant codec is now an
in-scope common quant codec-shape gap for Task 87, not a preferred
Stop Condition that defers DiskANN out of Task 87. Keep this file as
historical source-audit context unless Task 87 later spins out a
reviewer-approved prerequisite-only codec surface.

## Why

The original version of this follow-up assumed Task 87 scoped
candidate-batched kernel work to **TurboQuant no-QJL 4-bit** across all
AMs and allowed a DiskANN Stop Condition. That assumption is now stale.

The current `ec_diskann` search-code surface still does not expose a
TurboQuant search codec:

- `DiskannBuildCodec` has `PqFastScan` and `RaBitQ` variants.
- `DiskannPreparedPrefilter` has binary-sidecar, grouped-PQ, and RaBitQ
  branches.
- There is no direct TurboQuant no-QJL 4-bit prefilter branch equivalent
  to the SPIRE, IVF, and HNSW TurboQuant scoring hooks.

Task 87 now owns both the `CandidateBatch` data-flow abstraction and the
common quant codec shape that should make adding TurboQuant to DiskANN a
registration step rather than bespoke DiskANN-only plumbing.

## Goal

Historical goal: decide and, if feasible, land an
on-disk-format-neutral DiskANN TurboQuant no-QJL 4-bit search-code
surface.

Current coordination: Task 87 should either land this through the
common quant codec shape or, if that surface is too large, create a
narrow prerequisite slice for the common codec trait/enum itself. It
should not use this file as evidence that DiskANN can be skipped.

## Scope

### In scope

1. Audit DiskANN metadata, tuple, build, insert, scan, and diagnostics
   surfaces for the minimum changes required to add a
   `VAMANA_SEARCH_CODEC_TURBOQUANT`-style search codec.
2. If narrow:
   - add the metadata discriminator;
   - encode TurboQuant no-QJL 4-bit search codes at build/insert time;
   - prepare the matching TurboQuant query scorer;
   - score DiskANN prefilter candidates through the TurboQuant no-QJL
     4-bit LUT path;
   - keep tuple layout and existing grouped-PQ/RaBitQ indexes
     backward-compatible.
3. Document how Task 87 Phase 4 consumes the resulting surface through
   the common quant codec shape.

### Out of scope

- Candidate batching itself. Task 87 owns `CandidateBatch` integration.
- TQ+ calibration. Task 89 owns TQ+ validation and format design.
- New SIMD kernels.
- Replacing grouped-PQ or RaBitQ DiskANN defaults.

## Validation Gate

If code lands:

1. Existing grouped-PQ and RaBitQ DiskANN pg_test surfaces pass.
2. TurboQuant DiskANN build/scan smoke passes under PG18.
3. Recall is byte-equal or within the documented TurboQuant baseline
   tolerance against the scalar/no-batch TurboQuant DiskANN path.
4. Storage compatibility is documented: old indexes read unchanged; new
   metadata discriminator is explicit and versioned.
5. No new `unsafe` outside existing AM/quantizer boundaries.

If code does not land inside Task 87, the acceptable escape hatch is a
reviewer-approved prerequisite-only common codec surface, not a broad
DiskANN deferral.

## Coordination

- No longer blocks Task 87 DiskANN integration by default; Task 87 owns
  the DiskANN common-codec gap.
- Must not be cited as a reason to skip DiskANN grouped-PQ/RaBitQ/
  binary-sidecar batch routing where those paths are batch-shaped.
- Coordinates with Task 89 only if the chosen DiskANN TurboQuant storage
  shape would affect future TQ+ calibration metadata.
