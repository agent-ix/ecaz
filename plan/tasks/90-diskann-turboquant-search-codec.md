# Task 90: DiskANN TurboQuant Search Codec Prerequisite

Status: proposed (2026-06-08)
Owner: coder (to be assigned). One coder, one branch.
Priority: 2 (Task 87 DiskANN prerequisite)

## Why

Task 87 scopes candidate-batched kernel work to **TurboQuant no-QJL
4-bit** across all AMs. The current `ec_diskann` search-code surface does
not expose a TurboQuant search codec:

- `DiskannBuildCodec` has `PqFastScan` and `RaBitQ` variants.
- `DiskannPreparedPrefilter` has binary-sidecar, grouped-PQ, and RaBitQ
  branches.
- There is no direct TurboQuant no-QJL 4-bit prefilter branch equivalent
  to the SPIRE, IVF, and HNSW TurboQuant scoring hooks.

Task 87 reviewer feedback accepted the shared `CandidateBatch` shape but
blocked Phase 2 until the DiskANN scope ambiguity was resolved. This task
is the explicit follow-up for enabling or rejecting a DiskANN TurboQuant
search codec without silently broadening Task 87 to grouped-PQ or RaBitQ.

## Goal

Decide and, if feasible, land an on-disk-format-neutral DiskANN
TurboQuant no-QJL 4-bit search-code surface that Task 87 can batch in a
later DiskANN slice.

If the codec is not narrow and format-neutral, file a Stop Condition
packet explaining why DiskANN remains deferred from Task 87's TQ-only
kernel gate.

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
3. If not narrow:
   - land a Stop Condition packet with source evidence and a follow-up
     design recommendation.
4. Document how Task 87 Phase 4 should consume the resulting surface.

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

If code does not land:

1. Stop Condition packet cites the exact source blockers.
2. Reviewer accepts the Stop Condition.
3. Task 87 Phase 4 remains deferred without reinterpreting grouped-PQ or
   RaBitQ as satisfying a TurboQuant no-QJL 4-bit gate.

## Coordination

- Blocks Task 87 DiskANN integration unless Task 87's accepted
  closeout explicitly uses a reviewer-approved Stop Condition for
  DiskANN.
- Must not block Task 87 SPIRE, IVF, or HNSW structural batching slices.
- Coordinates with Task 89 only if the chosen DiskANN TurboQuant storage
  shape would affect future TQ+ calibration metadata.
