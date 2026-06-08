# Task 87 Packet 023: Phase 7 Methodology And Closeout

## Scope

This packet is the final Task 87 Phase 7 closeout. It addresses the packet 021 cross-cutting feedback, supersedes packet 015's Phase 6 closeout matrix, and flips the task status to complete with this packet as the referenced closeout.

No Task 91 merge is required for this closeout. DiskANN and cross-AM `QuantCodec` migration remain handed off to Task 91 by packets 005 and 009.

## Changes

- Updated `plan/tasks/87-candidate-batched-scoring-across-ams.md` from `reopened-for-32-block-kernel` to `complete`, referencing this packet.
- Added `artifacts/methodology.md` to document the scoring-share reframe and HNSW zero-counter outcome.
- Added `artifacts/aggregate-matrix.md` with the Phase 7 real10k/50k/100k matrix and Task 87 counters.
- Added `artifacts/completion-audit.md` with the final acceptance audit.
- Added HNSW reloptions logs showing the current HNSW real-corpus profiles are source-backed, not TurboQuant FullLut surfaces.

## Closeout Claim

Task 87 closes on this scoped claim:

- `CandidateBatch` plumbing is present across the accepted AM slices.
- The 32-candidate LUT kernel lives under `src/quant/` and has scalar differential coverage.
- SPIRE and IVF TurboQuant no-QJL measured cells route through the common candidate-batch scorer and reach the LUT32 kernel.
- Recall is preserved in all measured routed off/on cells.
- End-to-end latency improves on the routed TurboQuant matrix except for one documented real10k SPIRE p99 flat outlier (`+0.4%`).
- HNSW Phase 7 LUT32 evidence is stopped for this task because the current real-corpus HNSW profiles are not TurboQuant FullLut surfaces and report zero Task 87 counters.
- DiskANN remains out of Task 87 Phase 7 by the accepted Task 91 handoff.

## Evidence

- Packet 021 real10k evidence:
  `reviews/task-87/021-spire-leaf-lut32-batching/`
- Packet 022 real50k/real100k evidence:
  `reviews/task-87/022-phase7-50k-100k-counter-suite/`
- Superseding matrix:
  `artifacts/aggregate-matrix.md`
- Methodology and HNSW stop condition:
  `artifacts/methodology.md`
- Completion audit:
  `artifacts/completion-audit.md`

## Reviewer Notes

This packet intentionally uses the packet 021-approved methodology reframe instead of adding off-path scalar scorer counters. The direct scalar-off-path-vs-LUT32-on-path scoring-share factor is not claimed; the matrix reports on-path scorer counters and end-to-end latency separately.

Please review whether the documented reframe and HNSW stop condition are acceptable as the final Task 87 closeout boundary.
