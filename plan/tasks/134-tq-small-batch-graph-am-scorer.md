# Task 134: TQ small-batch scorer for graph AMs (HNSW/DiskANN)

Status: **proposed** (2026-07-01). Owner: coder (to be assigned). Priority: P3
Follow-up to Task 125/126.

## Why

Task 126 raised `BLOCK_WIDTH` to 64. Task 125 evidence showed that any candidate
batch **smaller than the block width never hits the wide SIMD block kernel** —
it falls to the partial/tail path (DiskANN test flipped 32/7 → 0/39 kernel/scalar
at 39 candidates). Graph AMs (HNSW, DiskANN) score small, scattered neighbor
sets per frontier expansion, so they are exactly the workload the wide
LUT-streaming block kernel does **not** serve. The block-width lever is inert or
mildly harmful for them, and their scoring path is currently unoptimized and
unmeasured for the int16/shared-kernel changes.

## Scope

Investigate and, where justified, prototype ONE of:

- a small-batch / few-candidate scorer variant tuned for graph-traversal batch
  sizes (keeps the int16 LUT resident across a frontier's neighbor set); or
- batching neighbor candidates across multiple frontier nodes so the wide block
  kernel becomes applicable; or
- a source-grounded decision that the shared partial path is already adequate
  for graph AMs, with measurement.

## Out of Scope (hard)

- No new on-disk format/mode/reloption. No graph-traversal algorithm changes —
  scorer/candidate-marshalling path only.

## Required Evidence

- HNSW and DiskANN recall+latency A/B at 10k/50k/100k (`ecaz bench suite`) on at
  least Apple + Graviton, before/after the prototype.

## Gate / Exit Criteria

- A measurable graph-AM latency improvement at unchanged recall, or a
  source-grounded negative recording why small-batch scoring does not transfer.
- No new unsafe fn; no anti-pattern B.
