# Task 101 Packet 003: Review Follow-ups

Code checkpoint: `a808ee5c0c6ecd7a3fac9d8fbcf38bfd77dfa3cf` (`Address Task 101 width-cascade review cleanup`)

This packet addresses the actionable follow-ups from Task 101 packet 001/002 feedback.

## What Changed

- Removed the unused grouped-PQ migration leftovers:
  - `score_grouped_pq_tail_scalar`
  - `score_grouped_pq_batch_block32`
- Gated scalar reference wrappers that are now test-only:
  - `score_grouped_pq_scalar`
  - `score_lut_no_qjl_4bit_scalar`
- Updated the padded sub-32 partial helpers for grouped-PQ and lut32:
  - SIMD hosts still pad to 32 lanes and use the block path.
  - scalar-only hosts now score only live tail candidates directly and return `Isa::Scalar`, preserving honest scalar candidate timing and avoiding dead-lane work.

## Evidence

Artifacts: `reviews/task-101/003-review-followups/artifacts/`

- `cargo-test-candidate-batch.log`: `19 passed; 0 failed`
- `cargo-test-grouped-pq.log`: `35 passed; 0 failed`
- `cargo-test-lut32.log`: `6 passed; 0 failed`

Width histogram evidence is copied from Task 94 packet 027 into this packet:

- IVF grouped-PQ rows include `width_lt8`, `width_8_15`, `width_16_31`, `width_ge32`, with `scalar_candidates=0`.
- DiskANN grouped-PQ rows include the same width buckets, with `scalar_candidates=0`.

Task 87 compat evidence is citable from packet 002 exact-mode logs:

- `turboquant` contributes to Task 87 `lut32_flushes`.
- `turboquant_tiled_lut` and `turboquant_int8` keep direct rows while leaving `lut32_flushes=0`, so the compat aggregation remains scoped to the legacy TurboQuant line.

## Notes

Task 94 packet 027 is the source rerun packet for the width and latency evidence. It also records the stale-catalog diagnosis and the explicit local catalog refresh needed after the counter-schema change.
