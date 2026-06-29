# Review Request: Task 124 TQ Stage-2 A/B Suite and Partial Payload Loader

## Summary

This is a TurboQuant-focused Task 124 iteration checkpoint, not a closeout.

I ran the required 10k / 50k / 100k in-engine A/B matrix for the current
RaBitQ frontier -> TQ stage-2 -> exact/source f32 width-25 path. The result was
not promotable: recall matches f32, and TQ scoring is fully SIMD, but latency
was still behind or at best near parity because the index-side TQ sidecar reads
too much linked group/segment payload.

I then implemented a narrow scan optimization for that TQ sidecar path:
load only the requested survivor payloads for each direct rerank group and
avoid assembling a full group payload slab. The post-change suite keeps matched
recall and reduces 100k/nprobe64 TQ sidecar payload reads substantially, but it
still does not create a product win. The next TQ work should target payload
locality/layout rather than the block scorer.

## Code Changes

- `load_rerank_groups_by_header_tid` now groups requested survivor heap TIDs by
  direct rerank-group TID.
- Added a selected-payload loader for direct index-side rerank groups.
- Added range-copy logic so the hot path copies only requested payload slices
  instead of unrelated group bytes.
- Preserved the existing full-group loader and full-payload lookup path for
  fallback callers.

## Key Benchmark Findings

All cited artifacts are under `artifacts/manifest.md`.

Post-change recall still matches f32 at every measured cell:

- 10k: both `1.0000` at nprobe 32 and 64.
- 50k: both `0.9960` at nprobe 32 and `1.0000` at nprobe 64.
- 100k: both `0.9730` at nprobe 32 and `1.0000` at nprobe 64.

Post-change latency remains only near parity, not a win:

| Scale | nprobe | f32 p50 | TQ p50 | f32 p95 | TQ p95 | f32 p99 | TQ p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 32 | 0.78 ms | 0.71 ms | 0.88 ms | 0.81 ms | 1.01 ms | 1.08 ms |
| 10k | 64 | 1.26 ms | 1.22 ms | 1.38 ms | 1.41 ms | 1.41 ms | 1.60 ms |
| 50k | 32 | 2.42 ms | 2.40 ms | 2.67 ms | 2.69 ms | 2.79 ms | 2.89 ms |
| 50k | 64 | 4.65 ms | 4.67 ms | 4.99 ms | 5.04 ms | 5.23 ms | 5.19 ms |
| 100k | 32 | 4.79 ms | 5.00 ms | 5.23 ms | 5.42 ms | 5.64 ms | 6.15 ms |
| 100k | 64 | 8.81 ms | 9.06 ms | 9.20 ms | 9.38 ms | 9.66 ms | 9.68 ms |

100k/nprobe64 sidecar attribution improved but still shows the bottleneck:

- TQ payload bytes actually scored: `77,200`.
- Source f32 final bytes read: `153,600` versus f32 baseline source bytes
  `614,400`.
- Index-side TQ segment pages read: `361 -> 216`.
- Index-side TQ segment payload bytes read: `2,839,027 -> 1,748,632`.
- TQ payload decode time: `1202 us -> 514 us`.

The TQ scorer is not scalar in this lane. The post-change counter rows report
`scalar_candidates=0` and `width_ge32=100` for TurboQuant at every
scale/nprobe.

## Validation

- `cargo test -p ecaz am::ec_ivf::scan --lib --no-default-features --features pg18`
  - `test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 2194 filtered out`
- Release build and PG18 install were rerun before the post-change scan suite.
- `ecaz bench suite` post-change scan-only suite completed for 10k / 50k /
  100k recall, latency, and explain cells.

## Outcome / Next Work

Recommended outcome for this slice: keep the optimization as a measured
increment, but keep Task 124 open as **iterate**.

The space has been explored far enough to rule out the simple explanation that
TQ was slow because it was scalar. It is full SIMD here. The next optimization
should be a TQ payload-locality/layout slice: the current sidecar still walks
linked group segments and reads roughly 1.75 MB of segment payload to score
77.2 KB of TQ payload at 100k/nprobe64.
