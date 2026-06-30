# Review Request: Task 124 / 033 TQ Payload Prefetch

## Summary

This slice addresses required Task 124 lever 7: **prefetch / pipelining payload reads ahead of scoring**.

Change:

- Added an ignored no-QJL LUT32 scorer profiler that compares the original scorer path against a next-block payload prefetch variant in the same binary.
- Implemented a disabled `prefetch_next_block` path for measurement only.
- Left production scoring on the original unprefetched width cascade because the measured result is mixed/noisy.

This is a TurboQuant scorer-path measurement and attempted optimization. It reports no-QJL LUT32 scorer `ns/candidate`, not f32 comparison, storage, nprobe, or end-to-end latency.

## TQ-Internal Result

Primary profiler rows from `artifacts/tq-prefetch-profile.log` on local arm64/NEON:

| Width | Original ns/candidate | Prefetch ns/candidate | Delta |
| --- | ---: | ---: | ---: |
| 8 | 253.4 | 235.1 | -7.2% |
| 16 | 232.5 | 232.7 | +0.1% |
| 25 | 295.2 | 295.5 | +0.1% |
| 32 | 231.7 | 231.3 | -0.2% |
| 64 | 232.3 | 231.9 | -0.2% |
| 96 | 232.1 | 232.3 | +0.1% |
| 100 | 244.1 | 241.3 | -1.1% |
| 128 | 232.8 | 233.6 | +0.3% |

Conclusion: this lever was attempted and measured, but the next-block prefetch is not a durable production win. The useful rows are small/noisy except width 8, and width 7/128 regress in the same sweep. Production remains unprefetched.

## Validation

Passed:

- `cargo fmt --check`
- `cargo test --release --lib --features bench turboquant_lut_batch_matches_scalar_tail -- --nocapture`
- `ECAZ_TQ_BATCH_WIDTH_PROFILE_LOG=reviews/task-124/033-tq-prefetch-pipeline/artifacts/tq-prefetch-profile.log cargo test --release --lib --features bench task124_profile_tq_no_qjl_flush_widths -- --ignored --nocapture`

Artifact details are in `artifacts/manifest.md`.

## Task Status

This completes the prefetch/pipelining attempt for lever 7, with no production change kept. Task 124 is still open; the remaining required unattempted lever is dimension/subspace reduction.
