# Task 79 AWS RaBitQ Deferred Decode Benchmark

## Summary

This packet records the post-review AWS Graviton confirmation run for the Task
79 RaBitQ candidate-surface reduction work. The suite uses the accepted local
Task 79 shape: `rabitq`, `leaf_block_rows=16`, K3 recursive layout,
`top_graph_search_list_size=96`, `nprobe=96`, `rerank_width=25`, and deferred
leaf-segment decode with a global block cap sweep.

The recommended/default AWS point is `global1152`: recall@10 `0.9945`, p50
`35.199 ms`, p95 `36.203 ms`, and `3,672,619` candidates across 200 queries.
That is about `3.82x` faster p50 and `4.13x` faster p95 than the prior AWS
high-recall SPIRE reference from `benchmarks/task73-74-aws-spire-quality-overhead-refresh-20260531/`
(`134.458 ms` p50 / `149.487 ms` p95 at recall@10 `0.9975`).

AWS compute was stopped after the run. `cloud status` reports profile `1m` as
`paused`, and the EC2 status artifact shows both `ecaz-cloud-1m-db` and
`ecaz-cloud-1m-loader` in `stopped` state.

## Evidence

- Packet manifest: `artifacts/manifest.md`
- Suite config: `suite.json`
- Suite manifest: `artifacts/suite-manifest.json`
- Raw suite results: `artifacts/results.jsonl`
- Parsed report rows: `artifacts/results-report.jsonl`
- Suite run log: `artifacts/suite-run.log`
- Final shutdown status:
  - `artifacts/cloud-status-after-pause.log`
  - `artifacts/ec2-status-after-pause.log`
  - `artifacts/ec2-status-after-pause.txt`

## Result Table

| Preset | Candidate Sum | Recall@10 | Latency p50 | Latency p95 | Latency p99 | Production read p50/p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `global1024` | `3,264,695` | `0.9920` | `33.354 ms` | `34.217 ms` | `34.638 ms` | `29 / 30 ms` |
| `global1152` recommended | `3,672,619` | `0.9945` | `35.199 ms` | `36.203 ms` | `36.591 ms` | `31 / 32 ms` |
| `global1216` safe | `3,876,624` | `0.9950` | `35.866 ms` | `36.659 ms` | `36.912 ms` | `32 / 32 ms` |

## Notes

- The first AWS bench attempt used retained `postgres` database state and hit a
  stale extension SQL object surface. The successful run used a fresh
  `task79_aws` database with `CREATE EXTENSION ecaz`, avoiding destructive
  changes to retained state.
- The initial cloud install attempts raced cloud-init. Cloud-init later
  completed the bootstrap, and the successful suite used `/usr/local/bin/ecaz`
  on the remote host.
