# Task 121 packet 015 artifact manifest

## Packet

- Task bucket: `reviews/task-121/`
- Packet path: `reviews/task-121/015-phase2-local-100k-f8-clean-latency/`
- Head SHA: `0d6ab558cd08a8f9e8c7398fba745f612e841a32`
- Timestamp: 2026-06-23 17:05-17:41 America/Los_Angeles
- Lane: Task 121 Phase 2 local 100k f8 clean latency
- Fixture: local staged real corpus, 100k, 200-query latency sweep
- Storage format / quantizer: TurboQuant f8 route-stage candidate surface,
  `bits=4`, `profile=ec_spire`
- Rerank mode: latency-only KNN path; no rerank-stage recall measurement in this
  packet
- Isolation: local single PostgreSQL instance, one index per table
- Remote / multi-node: `remote=false`; this is not the local multi-node Phase 0
  lane and is not AWS

## Commands

Audit:

```text
target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite audit --config reviews/task-121/015-phase2-local-100k-f8-clean-latency/artifacts/suite-phase2-local-100k-f8-clean-latency.json
```

Run:

```text
target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-121/015-phase2-local-100k-f8-clean-latency/artifacts/suite-phase2-local-100k-f8-clean-latency.json --manifest-output reviews/task-121/015-phase2-local-100k-f8-clean-latency/artifacts/suite-phase2-local-100k-f8-clean-latency-manifest.json --results-output reviews/task-121/015-phase2-local-100k-f8-clean-latency/artifacts/suite-phase2-local-100k-f8-clean-latency-results.jsonl --log-file reviews/task-121/015-phase2-local-100k-f8-clean-latency/artifacts/suite-phase2-local-100k-f8-clean-latency.log
```

## Artifacts

- `suite-phase2-local-100k-f8-clean-latency.json`
  - SuiteConfig for two clean latency steps.
  - Sweep: `24,32,48,64,96`
  - `concurrency=1`, `iterations=100`, `cache_state=post_pipeline_warm`
- `suite-phase2-local-100k-f8-clean-latency-audit.log`
  - Audit result: `[suite:task121-phase2-local-100k-f8-clean-latency] audit passed: 2 steps`
- `suite-phase2-local-100k-f8-clean-latency.log`
  - Suite runner log.
  - Both steps completed successfully.
- `suite-phase2-local-100k-f8-clean-latency.script.log`
  - Terminal capture for the suite run.
- `suite-phase2-local-100k-f8-clean-latency-manifest.json`
  - Structured suite manifest.
  - Step `latency-100k_b0_tr10_f8`: succeeded, duration `960710 ms`.
  - Step `latency-100k_b1_tr50_f8`: succeeded, duration `1306978 ms`.
- `suite-phase2-local-100k-f8-clean-latency-results.jsonl`
  - Structured latency result rows.
- `latency-100k_b0_tr10_f8.log`
  - Clean cache-warm baseline latency log.
- `latency-100k_b1_tr50_f8.log`
  - Clean cache-warm candidate latency log.
- `summary-100k-clean-latency.md`
  - Compact summary table and interim interpretation.

## Key result lines

Baseline `t121_s2_100k_b0_tr10_f8`:

| nprobe | count | mean | p50 | p95 | p99 | cache_state |
|---:|---:|---:|---:|---:|---:|---|
| 24 | 100 | 827.2 ms | 821.6 ms | 930.3 ms | 965.3 ms | post_pipeline_warm |
| 32 | 100 | 1108.2 ms | 1102.9 ms | 1204.8 ms | 1269.4 ms | post_pipeline_warm |
| 48 | 100 | 1765.9 ms | 1767.4 ms | 1935.1 ms | 1992.1 ms | post_pipeline_warm |
| 64 | 100 | 2348.1 ms | 2326.1 ms | 2526.3 ms | 2707.7 ms | post_pipeline_warm |
| 96 | 100 | 3540.3 ms | 3538.6 ms | 3718.6 ms | 3746.3 ms | post_pipeline_warm |

Candidate `t121_s2_100k_b1_tr50_f8`:

| nprobe | count | mean | p50 | p95 | p99 | cache_state |
|---:|---:|---:|---:|---:|---:|---|
| 24 | 100 | 1431.2 ms | 1420.0 ms | 1713.1 ms | 2094.7 ms | post_pipeline_warm |
| 32 | 100 | 1766.6 ms | 1776.2 ms | 1956.4 ms | 1992.1 ms | post_pipeline_warm |
| 48 | 100 | 2498.9 ms | 2508.0 ms | 2774.4 ms | 2945.9 ms | post_pipeline_warm |
| 64 | 100 | 3146.5 ms | 3129.7 ms | 3502.1 ms | 3548.5 ms | post_pipeline_warm |
| 96 | 100 | 4208.9 ms | 4193.1 ms | 4455.4 ms | 4562.2 ms | post_pipeline_warm |

## Interpretation

This packet provides the missing clean-latency evidence for the 100k f8
baseline/candidate pair. The b1/tr50 candidate carries a clear latency cost at
fixed nprobe: p50 is 1.73x baseline at nprobe 24, 1.61x at nprobe 32, 1.42x at
nprobe 48, 1.35x at nprobe 64, and 1.18x at nprobe 96.

This is not a final Task 121 verdict. The remaining Phase 2 matrix and Phase 3
scan-efficiency A/B are still required before promotion or closeout.
