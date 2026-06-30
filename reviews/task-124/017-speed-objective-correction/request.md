# Task 124 Objective Correction: Reopen For TQ Speed

## Correction

Packet `016-closeout-shelve` closed Task 124 too narrowly. It evaluated whether
TurboQuant stage-2 beat the current RaBitQ + f32 product baseline. The user
clarified that the immediate goal is to **improve TurboQuant speed**.

This packet supersedes the closeout decision. Task 124 is reopened for a
TQ-speed-focused implementation slice.

## What Remains Valid

The evidence from packets 001-015 is still useful:

- the TQ hot path is full SIMD/NEON, not scalar;
- the in-engine stage-2 path and counters exist;
- final15 is the best measured final exact width so far;
- selected payload slabs produced a measured TQ speed improvement;
- top-k fusion, compact headers, and direct-slot rerank did not help enough and should not be repeated unchanged.

## Revised Acceptance For The Next Slice

The next packet should answer a narrower speed question:

- What is the current best TQ configuration?
- What code change was made specifically to reduce TQ latency or TQ materialization/scoring overhead?
- What is the TQ-before/TQ-after delta on the same fixture?
- Did recall remain acceptable for the measured TQ configuration?
- Did TQ scorer counters remain SIMD with `scalar_candidates=0`?

RaBitQ + f32 may stay in the benchmark as context, but it is not the only success
criterion for the next Task 124 slice.
