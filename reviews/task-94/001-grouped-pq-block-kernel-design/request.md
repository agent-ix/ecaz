---
agent: coder-1
role: coder
model: gpt-5
date: 2026-06-09
seq: 01
---

# Task 94 Phase 1 Review Request — Grouped-PQ Block Kernel Design

## Scope

This is the Phase 1 design packet for Task 94. It contains no code changes.

The proposed implementation path is the ADR-076 32-wide grouped-PQ/PqFastScan
kernel under `src/quant/grouped_pq_block/`, registered only through
`QuantCodec::score_ip_batch`.

## Artifacts

- `artifacts/manifest.md`
- `artifacts/layout-audit.md`
- `artifacts/phase1-design.md`
- `artifacts/bench-suite-emitter-plan.md`

## Reviewer Questions

1. Approve `src/quant/grouped_pq_block/` as the module path despite the task
   file's older `pq_fastscan32` wording?
2. Approve the first AVX2 implementation using f32 gather from the existing
   row-major LUT, with byte/shuffle repacking deferred unless measurements
   require it?
3. For HNSW, approve Phase 6 as "register where the scan loop exposes a natural
   candidate batch; otherwise document scalar-only HNSW rather than forcing a
   risky scan-loop reshaping in Task 94"?

## Validation

Design-only packet. No tests, benches, CI, or AWS runs were performed.
