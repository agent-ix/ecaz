# Task 29e: DiskANN Rerank Cleanup Evidence

> **MULTI-NODE MEASUREMENT RULE (NON-NEGOTIABLE).** Any decision about
> distributed behavior — latency, recall, storage, or overhead — MUST be measured
> on a multi-node configuration. A single-node / single-instance arm is NEVER
> acceptable as the basis for a decision about a distributed algorithm; its only
> permitted use is a clearly labeled baseline that quantifies distribution
> overhead. Label every reported number with its arm's node count. See
> AGENTS.md → "Distributed Measurement: Multi-Node Arms Only".

Status: **complete on `main`** (status fixup 2026-05-31) — the rerank cleanup
slice landed; the packet records this as a code-shape cleanup, not a material
latency win. Closeout packet:
`reviews/task-29e/001-11110-task29e-rerank-borrowed-simd/`.
Owner: coder1 / runtime-index track

## Goal

Record the post-landing DiskANN rerank cleanup slice that followed Task 29d.
The kept change made exact heap rerank score borrowed `ecvector` datum slices
and reused the same dispatched inner-product helper as build. The evidence
shows this is a code-shape cleanup, not a material latency win.

## Evidence

- Review bucket: `reviews/task-29e/`
- Packet: `reviews/task-29e/001-11110-task29e-rerank-borrowed-simd/`

## Disposition

The cleanup may remain landed, but the rejected local experiments in the packet
are not active follow-up work. Further low-L DiskANN latency work should open a
new task or explicit Task 29 follow-up rather than continuing under 29e by
default.
