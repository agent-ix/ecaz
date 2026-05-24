# Task 51 AWS RaBitQ8 Sidecar Full Sweep

- Timestamp: `2026-05-24T03:00:25Z`
- Branch: `aws-optimization-ivf-rabitq-spire`
- Head SHA: `c00e93cc9`
- Task bucket: `reviews/task-51`
- Benchmark packet: `benchmarks/task51-aws-rabitq8-sidecar-full-sweep`
- Scope: AWS 1M IVF/RaBitQ sidecar-only sweep
- Variants: `rabitq8`, `rabitq8ls`, `rabitq8c3`, `rabitq8c4`
- Excluded: vchord, pgvectorscale/DiskANN, unchanged comparator reruns
- AWS profile: `10k-medium`
- AWS shape: DB `m8g.2xlarge`, loader `c8g.medium`
- Preserved snapshot: `snap-0b72153293b0b749b`
- Surface isolation: preserved 990k corpus/query tables and IVF/RaBitQ index; sidecar measurement tables built under the same benchmark prefix

## Outcome

The AWS sidecar sweep completed successfully and produced four benchmark rows.

The run used `ecaz cloud bench` with SuiteConfig `suite.json`, database `tqvector_bench`, `nprobe=128`, `candidate_k=50`, `queries=200`, `concurrency=1`, and `tid-sorted` sidecar reads.

The sidecar step completed in `1,669,711 ms` and wrote:

- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl`
- `artifacts/sidecar-1m-rabitq8-new-variants-k50-q200-c1-tid-sorted.log`
- `artifacts/precheck-preserved-1m-ivf-rabitq.log`

Compute was then paused with `ecaz cloud pause --profile 10k-medium`.

Final observed status:

```text
profile:  10k-medium
state:    paused
db:       10.42.1.80 (i-076683d54d878df15)
bucket:   ecaz-cloud-10k-medium-a02e4aea
snapshot: snap-0b72153293b0b749b
cost:     ~$0.00/hr running, ~$8.00/mo retained storage
```

## Results

All rows use `read_mode=tid-sorted`, `sidecar_bytes_per_vector=1548`, `sidecar_size=1.43 GiB`, and `candidate_sql_p50=1759.456 ms`.

| variant | recall@10 | ndcg@10 | sidecar_io_p50 | sidecar_p50 | total_bound_p50 | sidecar_io_p95 | total_bound_p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `rabitq8` | 0.9455 | 0.9989 | 21.859 ms | 22.085 ms | 1773.077 ms | 144.001 ms | 4329.259 ms |
| `rabitq8ls` | 0.9405 | 0.9988 | 21.952 ms | 22.177 ms | 1773.122 ms | 38.737 ms | 4328.835 ms |
| `rabitq8c3` | 0.9700 | 0.9990 | 22.408 ms | 22.633 ms | 1773.346 ms | 137.470 ms | 4330.696 ms |
| `rabitq8c4` | 0.9800 | 0.9991 | 17.165 ms | 17.390 ms | 1768.628 ms | 35.135 ms | 4327.225 ms |

`rabitq8c4` had the best recall and the lowest p50/p95 sidecar I/O in this sweep.

## Evidence

- `suite.json`: sidecar-only SuiteConfig for the new RaBitQ8 variants.
- `artifacts/precheck-preserved-1m-ivf-rabitq.log`: verifies `tqvector_bench`, PostgreSQL 18.3, 990000 corpus rows, 10000 query rows, and the preserved `ec_ivf` RaBitQ index.
- `artifacts/suite-run.log`: suite execution log; both steps succeeded.
- `artifacts/suite-manifest.json`: structured suite manifest with step status and durations.
- `artifacts/results.jsonl`: parsed result rows for the four variants.
- `artifacts/sidecar-1m-rabitq8-new-variants-k50-q200-c1-tid-sorted.log`: full sidecar result table.

## Follow-Up

The completed run used the pre-`0429af2ab` remote binary, so it still rebuilt all sidecar measurement tables and held all four sidecar encodings during the step. Commit `0429af2ab` has since reduced future run pressure by building one variant at a time and disabling forced sidecar table rebuilds in the suite config.
