# Task 121 review request: Phase 2 local 100k f8 clean latency checkpoint

## Scope

This packet responds to the reviewer gap in
`reviews/task-121/012-phase2-local-50k-100k-b0-b1-run/feedback/2026-06-23-01-reviewer.md`:
the prior Phase 2 pipeline `mean q-time` rows were funnel-instrumented and not
decision-grade latency. This packet adds a clean `latency` suite step for two
100k f8 cells:

- Baseline: `t121_s2_100k_b0_tr10_f8`
- Current candidate: `t121_s2_100k_b1_tr50_f8`

This is local-only evidence on the existing PG18 local benchmark database. It is
not AWS evidence. It is also not the Phase 0 local multi-node lane; both measured
tables are in one local PostgreSQL instance and this packet should not be used
as multi-node closeout evidence.

## Validation

- Audit:
  `target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite audit --config reviews/task-121/015-phase2-local-100k-f8-clean-latency/artifacts/suite-phase2-local-100k-f8-clean-latency.json`
- Run:
  `target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-121/015-phase2-local-100k-f8-clean-latency/artifacts/suite-phase2-local-100k-f8-clean-latency.json --manifest-output reviews/task-121/015-phase2-local-100k-f8-clean-latency/artifacts/suite-phase2-local-100k-f8-clean-latency-manifest.json --results-output reviews/task-121/015-phase2-local-100k-f8-clean-latency/artifacts/suite-phase2-local-100k-f8-clean-latency-results.jsonl --log-file reviews/task-121/015-phase2-local-100k-f8-clean-latency/artifacts/suite-phase2-local-100k-f8-clean-latency.log`

Both suite steps succeeded. The suite ran cache-warm `bench latency` with
`concurrency=1`, `iterations=100`, `bits=4`, `seed=42`, and nprobe sweep
`24,32,48,64,96`.

## Result

Clean latency shows that the b1/tr50 f8 candidate is materially slower than the
b0/tr10 f8 baseline at the same nprobe, especially at lower nprobe values:

| nprobe | b0_tr10_f8 p50 | b0_tr10_f8 p95 | b1_tr50_f8 p50 | b1_tr50_f8 p95 | p50 ratio |
|---:|---:|---:|---:|---:|---:|
| 24 | 821.6 ms | 930.3 ms | 1420.0 ms | 1713.1 ms | 1.73x |
| 32 | 1102.9 ms | 1204.8 ms | 1776.2 ms | 1956.4 ms | 1.61x |
| 48 | 1767.4 ms | 1935.1 ms | 2508.0 ms | 2774.4 ms | 1.42x |
| 64 | 2326.1 ms | 2526.3 ms | 3129.7 ms | 3502.1 ms | 1.35x |
| 96 | 3538.6 ms | 3718.6 ms | 4193.1 ms | 4455.4 ms | 1.18x |

This closes the immediate clean-latency gap for these two 100k f8 cells, but it
does not complete Phase 2 or Phase 4. Still owed before a Pareto/promote call:

- 50k b2/b4 cells
- full 100k recall matrix beyond the current baseline/candidate slice
- clean latency for any additional finalist cells
- Phase 3 scan-efficiency A/B and Phase 4 verdict

## Artifacts

- `artifacts/manifest.md`
- `artifacts/suite-phase2-local-100k-f8-clean-latency.json`
- `artifacts/suite-phase2-local-100k-f8-clean-latency-audit.log`
- `artifacts/suite-phase2-local-100k-f8-clean-latency.log`
- `artifacts/suite-phase2-local-100k-f8-clean-latency.script.log`
- `artifacts/suite-phase2-local-100k-f8-clean-latency-manifest.json`
- `artifacts/suite-phase2-local-100k-f8-clean-latency-results.jsonl`
- `artifacts/latency-100k_b0_tr10_f8.log`
- `artifacts/latency-100k_b1_tr50_f8.log`
- `artifacts/summary-100k-clean-latency.md`
