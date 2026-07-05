# Task 87 Packet 005: DiskANN Stop Condition

## Summary

This packet asks reviewers to accept the pre-declared Task 87 Phase 4
DiskANN Stop Condition from packet 002. No code changes are included.

Task 87 scopes AM batching to TurboQuant no-QJL 4-bit. The current
DiskANN search-code surface exposes grouped-PQ and RaBitQ paths, but no
TurboQuant no-QJL 4-bit search codec or prefilter scorer. Routing those
other quantizers through `CandidateBatch` would broaden Task 87 beyond
the accepted scope and would not satisfy the TurboQuant gate.

## Evidence

See `artifacts/source-audit.md` and `artifacts/manifest.md`.

- `DiskannBuildCodec` currently has `PqFastScan` and `RaBitQ` variants.
- `DiskannPreparedPrefilter` currently has `BinarySidecar`, `GroupedPq`,
  and `RaBitQ` scoring branches.
- There is no DiskANN branch equivalent to the SPIRE/IVF/HNSW
  TurboQuant no-QJL 4-bit LUT scorer.
- `plan/tasks/90-diskann-turboquant-search-codec.md` owns the follow-up
  decision for adding or rejecting a narrow DiskANN TurboQuant search
  codec.

## Request

Please review whether this is an acceptable Task 87 Phase 4 Stop
Condition:

- Task 87 does not reinterpret grouped-PQ or RaBitQ as satisfying the
  TurboQuant no-QJL 4-bit CandidateBatch requirement.
- Task 87 can continue with the SPIRE, IVF, and HNSW structural slices.
- DiskANN remains deferred to Task 90 unless that prerequisite lands
  before Task 87 closeout.

## Validation

No tests were run for this packet because it is source-audit and review
scope documentation only.
