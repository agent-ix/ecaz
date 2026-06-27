# Task 123 Phase A Artifact Manifest

- Head SHA: `700f5de86782d190abf0a1bf0fd67954394a397f`
- Task bucket: `reviews/task-123/001-phase-a-latency-floor-decomposition`
- Timestamp: `2026-06-27T15:30:45Z`
- Lane: local PG18, database `tqvector_bench_task121`, socket `/tmp`, port `28818`
- Backend: release `ecaz.so` at `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`, sha256 `f35657b0d65ecd87ab80db780efbed51d5d0acc4234a099f61bb02b079ab9cd2`
- Fixture: staged real corpus 10k / 50k / 100k, existing Task 121 Phase 3 SPIRE surfaces
- Surface isolation: one table and one `ec_spire` index per scale, reused from Task 121:
  - `t121_s3_10k_b4_tr50_f8_b64`
  - `t121_s3_50k_b4_tr50_f8_b64`
  - `t121_s3_100k_b4_tr50_f8_b64`
- Index reloptions: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=4`, `training_sample_rows=50000`, `storage_format=rabitq`; see `flat-floor-plan.log`.
- Rerank mode: SPIRE index default, `rerank_width=0` in cost snapshot.
- Suite config: `task123-phase-a-suite.json`
- Suite manifest: `suite-manifest.json`
- Normalized results: `suite-results.jsonl`

## Commands

Audit:

```sh
/home/peter/dev/ecaz/target/debug/ecaz bench suite audit \
  --config reviews/task-123/001-phase-a-latency-floor-decomposition/artifacts/task123-phase-a-suite.json
```

Run:

```sh
/home/peter/dev/ecaz/target/debug/ecaz bench suite run \
  --config reviews/task-123/001-phase-a-latency-floor-decomposition/artifacts/task123-phase-a-suite.json \
  --database tqvector_bench_task121 \
  --host /tmp \
  --port 28818 \
  --manifest-output reviews/task-123/001-phase-a-latency-floor-decomposition/artifacts/suite-manifest.json \
  --results-output reviews/task-123/001-phase-a-latency-floor-decomposition/artifacts/suite-results.jsonl \
  --allow-debug-backend
```

The suite preflight recorded the installed backend as release despite the CLI flag.

## Artifacts

- `flat-floor-plan.sql`: SQL used by the raw suite step to prove the flat-floor plan disables index scans.
- `flat-floor-plan.log`: raw plan/precheck output. Shows `enable_seqscan=on`, `enable_indexscan=off`, `enable_indexonlyscan=off`, `enable_bitmapscan=off`, and sequential scans for 10k / 50k / 100k.
- `latency-flat-floor-10k.log`, `latency-flat-floor-50k.log`, `latency-flat-floor-100k.log`: exact flat seq-scan latency floors.
- `latency-spire-10k-nprobe-8-96.log`, `latency-spire-50k-nprobe-8-96.log`, `latency-spire-100k-nprobe-8-96.log`: clean SPIRE latency at nprobe 8 and 96.
- `spire-pipeline-10k-nprobe-8-96.log`, `spire-pipeline-50k-nprobe-8-96.log`, `spire-pipeline-100k-nprobe-8-96.log`: pipeline counters, query metrics, recall, local store overlap, and cost snapshots.
- `funnel-10k-nprobe-8-96.jsonl`, `funnel-50k-nprobe-8-96.jsonl`, `funnel-100k-nprobe-8-96.jsonl`: per-query funnel counters and timing fields.
- `stage-containment-10k-nprobe-8-96.jsonl`, `stage-containment-50k-nprobe-8-96.jsonl`, `stage-containment-100k-nprobe-8-96.jsonl`: per-query route/stage truth-containment records.

## Key Results

Flat floor and SPIRE clean latency, from `suite-results.jsonl`:

| Scale | Flat p50 | Flat p95 | SPIRE nprobe 8 p50 / p95 | SPIRE nprobe 96 p50 / p95 | nprobe 96 p50 ratio |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k | 29.4 ms | 51.6 ms | 103.8 / 148.1 ms | 496.2 / 560.0 ms | 16.9x |
| 50k | 80.2 ms | 168.7 ms | 428.2 / 542.2 ms | 2159.5 / 2634.7 ms | 26.9x |
| 100k | 223.3 ms | 354.3 ms | 965.9 / 1332.6 ms | 5483.0 / 6233.7 ms | 24.6x |

Pipeline query metrics, from `spire-pipeline-*-nprobe-8-96.log`:

| Scale | nprobe | Recall@10 | Pipeline p50 | Pipeline p95 |
| --- | ---: | ---: | ---: | ---: |
| 10k | 8 | 0.9875 | 87.624 ms | 113.709 ms |
| 10k | 96 | 1.0000 | 407.203 ms | 442.812 ms |
| 50k | 8 | 0.9938 | 405.845 ms | 464.253 ms |
| 50k | 96 | 1.0000 | 2131.257 ms | 2336.708 ms |
| 100k | 8 | 0.9375 | 936.970 ms | 1175.987 ms |
| 100k | 96 | 1.0000 | 4915.933 ms | 5536.616 ms |

Route-stage containment, aggregated from `stage-containment-*-nprobe-8-96.jsonl`:

| Scale | nprobe | Route containment | Final recall basis |
| --- | ---: | ---: | ---: |
| 10k | 8 | 316 / 320 = 98.75% | 316 / 320 = 98.75% |
| 10k | 96 | 320 / 320 = 100.00% | 320 / 320 = 100.00% |
| 50k | 8 | 318 / 320 = 99.375% | 318 / 320 = 99.375% |
| 50k | 96 | 320 / 320 = 100.00% | 320 / 320 = 100.00% |
| 100k | 8 | 300 / 320 = 93.75% | 300 / 320 = 93.75% |
| 100k | 96 | 320 / 320 = 100.00% | 320 / 320 = 100.00% |

Local-store scan/candidate volume, from `spire-pipeline-*-nprobe-8-96.log` and `funnel-*-nprobe-8-96.jsonl`:

| Scale | nprobe | Candidates/query | Object bytes/query | Leaf read ms/query | Candidate score ms/query | Heap append ms/query |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 8 | 3,574 | 2.9 MiB | 1.1 | 1.9 | 0.7 |
| 10k | 96 | 37,861 | 30.4 MiB | 10.5 | 19.5 | 11.0 |
| 50k | 8 | 16,326 | 13.1 MiB | 4.4 | 8.4 | 4.3 |
| 50k | 96 | 186,824 | 149.8 MiB | 96.2 | 95.4 | 71.2 |
| 100k | 8 | 31,330 | 25.1 MiB | 8.2 | 15.9 | 9.4 |
| 100k | 96 | 378,986 | 303.7 MiB | 210.2 | 199.0 | 164.0 |

## Interpretation

Phase A fails the high-recall flat-floor gate. The recall-1.0 nprobe 96 SPIRE path is 16.9x / 26.9x / 24.6x slower than the flat exact p50 at 10k / 50k / 100k. The loss is not introduced by candidate scoring or rerank: route containment equals final recall in every row. At 100k nprobe 96, SPIRE reads about 303.7 MiB of local-store objects and scores about 379k candidates per query, so the binding wall is the local scan/candidate path after route recovery, not route precision alone.
