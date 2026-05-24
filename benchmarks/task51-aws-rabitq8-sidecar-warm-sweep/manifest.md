# Task 51 AWS RaBitQ8 Sidecar Warm Sweep

- Branch: `aws-optimization-ivf-rabitq-spire`
- Task bucket: `reviews/task-51`
- Benchmark packet: `benchmarks/task51-aws-rabitq8-sidecar-warm-sweep`
- Scope: warm AWS 1M IVF/RaBitQ sidecar-only rerun
- Variants: `rabitq8`, `rabitq8ls`, `rabitq8c3`, `rabitq8c4`
- Excluded: vchord, pgvectorscale/DiskANN, unchanged comparator reruns
- AWS profile: `10k-medium`
- Preserved snapshot: `snap-0b72153293b0b749b`

## Intended Run

This packet reruns the new-sidecar sweep after the cold run in
`benchmarks/task51-aws-rabitq8-sidecar-full-sweep`.

Differences from the cold run:

- `warmup_queries=200` before timed candidate and sidecar metrics.
- `rebuild_sidecar_table=false` so complete sidecar measurement tables are reused.
- Remote binary includes the one-variant-at-a-time sidecar harness change from `0429af2ab`.

## Result

Status: succeeded.

This rerun answers the cold-cache concern from
`benchmarks/task51-aws-rabitq8-sidecar-full-sweep`: the earlier
`candidate_sql_p50=1759.456 ms` was not a steady-state IVF scan cost.
With `warmup_queries=200`, the same new-sidecar sweep reports
`candidate_sql_p50=35.095 ms`, matching the established warm 1M IVF
shape.

All cells used:

- Head SHA installed on AWS: `7325e3bb123924cd79ccfd09a55db6cebbb72c86`
- Suite config: `benchmarks/task51-aws-rabitq8-sidecar-warm-sweep/suite.json`
- Suite manifest: `artifacts/suite-manifest.json`
- Results: `artifacts/results.jsonl`
- Fixture: `real_1m_ivf_rabitq1_rerank`
- Corpus rows: `990000`
- Query rows available: `10000`
- Timed queries: `200`
- Warmup queries: `200`
- Candidate K: `50`
- K: `10`
- nprobe: `128`
- Read mode: `tid-sorted`
- Concurrency: `1`
- Sidecar table rebuild: `false`
- Isolated one-index-per-table surface: yes

| Variant | recall@k | ndcg@k | candidate_sql_p50 | sidecar_io_p50 | sidecar_p50 | total_bound_p50 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `rabitq8` | 0.9455 | 0.9989 | 35.095 ms | 0.184 ms | 0.409 ms | 35.511 ms |
| `rabitq8ls` | 0.9405 | 0.9988 | 35.095 ms | 0.178 ms | 0.403 ms | 35.517 ms |
| `rabitq8c3` | 0.9700 | 0.9990 | 35.095 ms | 0.179 ms | 0.405 ms | 35.515 ms |
| `rabitq8c4` | 0.9800 | 0.9991 | 35.095 ms | 0.180 ms | 0.406 ms | 35.506 ms |

## Evidence

- `artifacts/precheck-preserved-1m-ivf-rabitq.log`: PostgreSQL 18.3,
  corpus/query counts, IVF index reloptions, and the four preserved
  sidecar table row estimates.
- `artifacts/suite-manifest.json`: suite-run provenance and expanded
  command. The sidecar step includes `--warmup-queries 200`.
- `artifacts/suite-run.log`: suite execution log for the precheck and
  sidecar-rerank steps.
- `artifacts/warm-sidecar-1m-rabitq8-new-variants-k50-q200-warm200-c1-tid-sorted.log`:
  raw sidecar-rerank output.
- `artifacts/results.jsonl`: structured suite result rows.

## Cloud State

After artifact sync, profile `10k-medium` was paused:

- DB instance: `i-076683d54d878df15`
- Snapshot retained: `snap-0b72153293b0b749b`
- Running compute cost: `$0.00/hr`
- Retained storage estimate: `$8.00/mo`

## Notes

The explicit warmup makes this a page-cache-hot measurement for both
the IVF candidate query and the sidecar tables. It is the correct
apples-to-apples check for the steady-state IVF-vs-sidecar question,
but it should not be cited as a cold-start or cache-miss tail result.
