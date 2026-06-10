---
agent: coder-1
role: coder
model: gpt-5
date: 2026-06-09
seq: 03
---

# Task 94 Phase 3 Review Request — NEON Grouped-PQ Block Backend

## Scope

This packet implements the Phase 3 NEON backend for the grouped-PQ /
PqFastScan 32-candidate block kernel.

Code checkpoint:

- `16872ca0f Add grouped-PQ NEON block backend`

## What Changed

- `src/quant/grouped_pq_block/neon.rs` now has an aarch64 NEON backend instead
  of only a scalar fallback stub.
- The NEON backend scores four candidates at a time with a vector accumulator,
  preserving the same group-order accumulation per lane as
  `grouped_pq_score_f32`.
- Runtime NEON detection gates the backend; unsupported hosts still delegate to
  scalar and return `Isa::Scalar`.
- Added a test hook and conditional parity test for real NEON execution when
  the local host supports it.

## Validation

Local only:

```text
cargo test grouped_pq_block --lib
```

Result: 6 passed, 0 failed. See
`artifacts/test-grouped-pq-block.log`.

No CI, AWS, or benchmark runs were performed.

## Evidence Limits

The local host did not provide approved Graviton-4 runtime evidence in this
packet. The NEON parity test executes the real backend only on NEON-capable
aarch64 hosts; otherwise it returns early. Graviton-4 dispatch and measured
runtime vector-length evidence remain for the approved AWS/final evidence run.
