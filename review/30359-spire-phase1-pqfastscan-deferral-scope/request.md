# 30359 SPIRE Phase 1 PQ-FastScan Deferral Scope

## Request

Review the Phase 1 landing-scope clarification for scannable assignment payload
formats.

## Scope

- Updated Task 30 Phase 1 scope text.
- Clarified that TurboQuant and RaBitQ are the Phase 1 scannable formats.
- Clarified that RaBitQ is the compact scannable target for the Phase 1
  storage/recall/speed tradeoff.
- Moved populated PQ-FastScan wording to an explicit post-Phase-1 deferred
  item, not a Phase 1 landing blocker.

## Decision

Phase 1 should land with populated SPIRE scans for TurboQuant and RaBitQ.
Populated PQ-FastScan remains recognized and diagnosed, but build/scan support
is intentionally deferred until SPIRE persists grouped-PQ model metadata and
binds that metadata to the PQ-FastScan scorer.

Final storage, recall, and latency claims for the Phase 1 scannable formats
remain gated on the landing review packet's measurement evidence.

## Validation

- `git diff --check`

Docs-only change; no tests were run.

