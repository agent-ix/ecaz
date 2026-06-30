# Review Request: Task 124 / 032 TQ2 SIMD Scorer

## Summary

This slice addresses required Task 124 lever 5: **TQ2 with a real SIMD kernel**.

Change:

- Added `quant::qjl2_32`, a TurboQuant2 QJL scorer for one-MSE-bit plus one-QJL-sign-bit payloads.
- Implemented a candidate-parallel NEON octet/block scorer for TQ2, with scalar fallback and correctness tests against the existing pre-slice scorer.
- Added a `CandidateBatch` TQ2 QJL batch surface and routed IVF TurboQuant2 rerank batches through it.
- Preserved the prior fallback behavior for non-default/non-TQ2 TurboQuant bit widths.
- Added an ignored Task 124 scorer profiler that reports old per-payload TQ2 scoring vs the new TQ2 batch scorer in `ns/candidate`.

This is a TurboQuant scorer-path optimization. It reports TQ2 scorer `ns/candidate`, not f32 comparison, storage, nprobe, or end-to-end latency.

## TQ-Internal Result

Primary profiler deltas from `artifacts/tq2-qjl-profile.log` on local arm64/NEON:

| Width | Old per-payload ns/candidate | New TQ2 batch ns/candidate | Delta |
| --- | ---: | ---: | ---: |
| 8 | 2098.4 | 282.8 | -86.5% |
| 16 | 2100.6 | 281.8 | -86.6% |
| 25 | 2103.9 | 318.8 | -84.8% |
| 32 | 2114.9 | 285.1 | -86.5% |
| 64 | 2105.3 | 282.6 | -86.6% |
| 96 | 2106.5 | 280.6 | -86.7% |
| 100 | 2100.5 | 320.3 | -84.8% |
| 128 | 2108.6 | 280.6 | -86.7% |

The profiler confirms the new TQ2 scorer dispatches on `backend=neon`. Widths below 8 still carry scalar/call-bound overhead, but natural octet/block widths now use the TQ2 SIMD scorer instead of the old per-payload loop.

## Validation

Passed:

- `cargo fmt --check`
- `cargo test --release --lib --features bench qjl2 -- --nocapture`
- `cargo test --release --lib --features bench turboquant2 -- --nocapture`
- `ECAZ_TQ2_QJL_PROFILE_LOG=reviews/task-124/032-tq2-simd-scorer/artifacts/tq2-qjl-profile.log cargo test --release --lib --features bench task124_profile_tq2_qjl_flush_widths -- --ignored --nocapture`

Artifact details are in `artifacts/manifest.md`.

## Task Status

This completes the TQ2 SIMD scorer slice for lever 5. Task 124 is still open; at minimum, the remaining required unattempted levers are dimension/subspace reduction and prefetch/pipelining payload reads ahead of scoring.
