# Task 90: DiskANN TurboQuant Search Codec Prerequisite

Status: superseded by Task 91 (2026-06-08)
Owner: coder (to be assigned). One coder, one branch.
Priority: 2 (Task 87 DiskANN prerequisite)

2026-06-08 update: Task 91
`plan/tasks/91-cross-am-quantcodec-migration.md` supersedes this
standalone prerequisite. DiskANN's missing TurboQuant codec is Task 91
Phase 6 work, after the shared `QuantCodec` migration and per-AM parity
gates have landed. Keep this file as historical source-audit context.

## Why

The original version of this follow-up assumed Task 87 scoped
candidate-batched kernel work to **TurboQuant no-QJL 4-bit** across all
AMs and allowed a DiskANN Stop Condition. The DiskANN Stop Condition is
again valid Task 87 territory; the TurboQuant codec landing moved to
Task 91.

The current `ec_diskann` search-code surface still does not expose a
TurboQuant search codec:

- `DiskannBuildCodec` has `PqFastScan` and `RaBitQ` variants.
- `DiskannPreparedPrefilter` has binary-sidecar, grouped-PQ, and RaBitQ
  branches.
- There is no direct TurboQuant no-QJL 4-bit prefilter branch equivalent
  to the SPIRE, IVF, and HNSW TurboQuant scoring hooks.

Task 91 now owns the common quant codec shape that should make adding
TurboQuant to DiskANN a registration step rather than bespoke
DiskANN-only plumbing. Task 87 owns only the CandidateBatch data-flow
and batch-shaped scoring work on existing per-AM codec surfaces.

## Goal

Historical goal: decide and, if feasible, land an
on-disk-format-neutral DiskANN TurboQuant no-QJL 4-bit search-code
surface.

Current coordination: Task 91 should close this by reference once Phase
6 lands DiskANN TurboQuant search-codec support. Task 87 should not use
this file as evidence that DiskANN can be skipped entirely; Task 87's
accepted DiskANN Stop Condition remains the proper handoff.

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
3. Document how Task 91 Phase 6 exposes the resulting surface and how
   later Task 87-style batch kernels can consume it.

### Out of scope

- Candidate batching itself. Task 87 owns `CandidateBatch` integration
  on existing per-AM codec surfaces.
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

Task 90 closes by reference when Task 91 Phase 6 lands and is reviewed.

## Coordination

- No longer blocks Task 87 DiskANN integration by default; Task 91 owns
  the DiskANN common-codec gap.
- Must not be cited as a reason to skip DiskANN grouped-PQ/RaBitQ/
  binary-sidecar batch routing where those paths are batch-shaped.
- Coordinates with Task 89 only if the chosen DiskANN TurboQuant storage
  shape would affect future TQ+ calibration metadata.
