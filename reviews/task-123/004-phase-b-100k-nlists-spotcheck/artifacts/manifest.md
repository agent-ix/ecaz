# Task 123 Phase B 100k nlists Spot-Check Artifact Manifest

- Head SHA: `4920c474bcbaa022c68ff54c007bd3ba9c0c7a65`
- Task bucket: `reviews/task-123/004-phase-b-100k-nlists-spotcheck`
- Timestamp: `2026-06-27T16:31:23Z`
- Lane: local PG18, database `tqvector_bench_task121`, socket `/tmp`, port `28818`
- Fixture: staged real corpus 100k (`/home/peter/dev/ecaz/data/staged-current/ec_real_100k_*`)
- Surface isolation: one table and one `ec_spire` index per tested cell:
  - `t123_p4_100k_n1024_b0_tr50_f8`
  - `t123_p4_100k_n1024_b1_tr50_f8`
- Storage format / rerank mode: `storage_format=rabitq`, SPIRE default rerank (`rerank_width=0` in cost snapshot)
- Variant axis: `nlists=1024`, `boundary_replica_count in {0,1}`, `training_sample_rows=50000`, `recursive_fanout=8`
- Suite config: `task123-phase-b-100k-nlists-spotcheck-suite.json`
- Suite manifest: `suite-manifest.json`
- Normalized results: `suite-results.jsonl`

## Commands

Audit:

```sh
/home/peter/dev/ecaz/target/debug/ecaz bench suite audit \
  --config reviews/task-123/004-phase-b-100k-nlists-spotcheck/artifacts/task123-phase-b-100k-nlists-spotcheck-suite.json
```

Run:

```sh
/home/peter/dev/ecaz/target/debug/ecaz bench suite run \
  --config reviews/task-123/004-phase-b-100k-nlists-spotcheck/artifacts/task123-phase-b-100k-nlists-spotcheck-suite.json \
  --database tqvector_bench_task121 \
  --host /tmp \
  --port 28818 \
  --manifest-output reviews/task-123/004-phase-b-100k-nlists-spotcheck/artifacts/suite-manifest.json \
  --results-output reviews/task-123/004-phase-b-100k-nlists-spotcheck/artifacts/suite-results.jsonl \
  --allow-debug-backend
```

## Artifacts

- `load-100k-n1024-b0-tr50-f8.log`, `load-100k-n1024-b1-tr50-f8.log`: corpus load, encoding, and index build logs.
- `storage-100k-n1024-b0-tr50-f8.log`, `storage-100k-n1024-b1-tr50-f8.log`: storage size output.
- `latency-flat-floor-100k-repeat.log`: repeated 100k flat exact floor with index scans disabled.
- `latency-spire-100k-n1024-b0-nprobe-8-16-32.log`, `latency-spire-100k-n1024-b1-nprobe-8-16-32.log`: clean cache-warm SPIRE latency.
- `spire-pipeline-100k-n1024-b0-nprobe-8-16-32.log`, `spire-pipeline-100k-n1024-b1-nprobe-8-16-32.log`: query metrics, recall, route/candidate counters, and local-store overlap.
- `funnel-100k-n1024-b0-nprobe-8-16-32.jsonl`, `funnel-100k-n1024-b1-nprobe-8-16-32.jsonl`: per-query funnel counters and timing.
- `stage-containment-100k-n1024-b0-nprobe-8-16-32.jsonl`, `stage-containment-100k-n1024-b1-nprobe-8-16-32.jsonl`: per-query route/stage truth-containment records.

## Key Results

The repeated 100k flat exact floor measured p50/p95 `203.8 / 425.8 ms`.

Clean latency from `suite-results.jsonl`:

| Config | nprobe | p50 | p95 | Ratio vs repeated flat p50 |
| --- | ---: | ---: | ---: | ---: |
| n1024 b0 | 8 | 75.5 ms | 98.9 ms | 0.37x |
| n1024 b0 | 16 | 95.1 ms | 120.9 ms | 0.47x |
| n1024 b0 | 32 | 153.8 ms | 180.8 ms | 0.75x |
| n1024 b1 | 8 | 102.3 ms | 123.5 ms | 0.50x |
| n1024 b1 | 16 | 143.8 ms | 170.1 ms | 0.71x |
| n1024 b1 | 32 | 236.1 ms | 290.4 ms | 1.16x |

Route containment and final recall, aggregated from `stage-containment-*.jsonl`:

| Config | nprobe | Route containment | Final recall basis | Recall@10 |
| --- | ---: | ---: | ---: | ---: |
| n1024 b0 | 8 | 223 / 320 | 223 / 320 | 0.6969 |
| n1024 b0 | 16 | 256 / 320 | 256 / 320 | 0.8000 |
| n1024 b0 | 32 | 280 / 320 | 280 / 320 | 0.8750 |
| n1024 b1 | 8 | 251 / 320 | 251 / 320 | 0.7844 |
| n1024 b1 | 16 | 282 / 320 | 282 / 320 | 0.8812 |
| n1024 b1 | 32 | 298 / 320 | 298 / 320 | 0.9313 |

Candidate and local-store volume from `suite-results.jsonl`:

| Config | nprobe | Candidates/query | Object bytes/query |
| --- | ---: | ---: | ---: |
| n1024 b0 | 8 | 770 | 0.6 MiB |
| n1024 b0 | 16 | 1,458 | 1.1 MiB |
| n1024 b0 | 32 | 2,897 | 2.3 MiB |
| n1024 b1 | 8 | 1,619 | 1.3 MiB |
| n1024 b1 | 16 | 3,073 | 2.4 MiB |
| n1024 b1 | 32 | 5,984 | 4.7 MiB |

Storage from `storage-*.log` and `suite-results.jsonl`:

| Config | SPIRE index size | All indexes | Total table |
| --- | ---: | ---: | ---: |
| n1024 b0 | 89.8 MiB | 92.0 MiB | 1.6 GiB |
| n1024 b1 | 167.9 MiB | 170.1 MiB | 1.7 GiB |

## Interpretation

This spot-check confirms that finer leaves reduce scan volume and clean latency
dramatically, but they do not recover 100k route containment at the requested
low nprobes. The best tested cell, `nlists=1024,boundary=1,nprobe=32`, reaches
only `298/320 = 0.9313` recall while its clean p50 is already `236.1 ms`, above
the repeated flat p50 and still below the Phase A `nlists=128,b4,nprobe=8`
recall of `300/320 = 0.9375`.

Route containment equals final recall in every row, so the failed recall remains
a routing-containment deficit. The requested Phase B spot-check therefore does
not show a viable finer-`nlists` escape hatch toward the reviewer's proposed
`~0.99` recall at `~4-6x` flat.
