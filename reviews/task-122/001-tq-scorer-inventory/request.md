# Task 122 Packet 001: TQ Scorer Inventory

## Summary

This starts Task 122 on a latest-main branch by pinning the current TurboQuant
scorer topology before any behavior-changing optimization work.

The main correction is that latest `main` does not contain the Task 89 closeout
packet, so the Task 122 reference now points to the Task 89 branch/commit where
that packet exists. I also refreshed a stale comment in the shared TQ tiled-LUT
batch scorer: it now records through the width-cascade block scorer path rather
than the older scalar-only wording.

## Findings

- IVF normal TurboQuant estimator/rerank lanes already route no-QJL and QJL
  payload batches through shared `CandidateBatch` block scorers.
- IVF exact-dequant rerank remains scalar and should not be used as the first
  latency comparator for TQ-vs-RaBitQ pipeline work.
- SPIRE assignment and routed V2 leaf scoring already route TQ candidate batches
  through shared batch scoring, but the V2 leaf path still scores a full column
  batch before row materialization/truncation. That is the first clear Phase 2
  target after scorer parity.
- HNSW has TQ batch scoring for exact payload batches and scan-code prefilter
  surfaces, but the single-candidate scan scoring helper remains scalar for
  fallback/use-at-a-time sites.
- DiskANN TQ is no-QJL 4-bit only and already routes prefilter batches through
  the shared no-QJL block scorer.

## Evidence

See `artifacts/tq-scorer-inventory.md` and `artifacts/manifest.md`.

## Validation

No tests run. The code change is comment-only; the task/reference and inventory
edits are documentation.

## Next Slice

Start Phase 2 on SPIRE or IVF by adding counters around score/retain/materialize
boundaries, then use those counters to make a small fused materialization change
that can be A/B measured with `ecaz bench suite`.
