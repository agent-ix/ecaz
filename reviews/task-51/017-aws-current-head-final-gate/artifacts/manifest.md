# Artifact Manifest: AWS Current-HEAD IVF/RaBitQ Final Gate

- head SHA at packet creation: `902e8e066944d4cabfb26ee5cc9039b466856891`
- AWS host head SHA: `902e8e066944d4cabfb26ee5cc9039b466856891`
- task bucket: `reviews/task-51/`
- packet path: `reviews/task-51/017-aws-current-head-final-gate/`
- benchmark packet: `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/`
- lane: AWS current-head final gate, IVF and RaBitQ only
- fixture: preserved real DBpedia corpus, 990000 corpus rows, 10000 query rows
- restored snapshot: `snap-0e0632400184fadd4`
- instance shape: DB `m8g.2xlarge`, loader `c8g.medium`
- storage format: `rabitq`
- preserved index reloptions: `{quant_bits=1,rerank=heap_f32,rerank_width=50,storage_format=rabitq}`
- rerank mode: preserved in-index `heap_f32`, `rerank_width=50`; sidecar harness measured with `candidate_k=50`
- isolated one-index-per-table surface: yes for the preserved AWS table/index
- vchord / pgvectorscale: not run
- suite runner: `ecaz bench suite`
- suite command id: `70df8076-1c85-4481-b1c9-a3e8bdbd7f88`
- suite status: `Success`, response code `0`, elapsed `PT31M14.927S`
- post-run stack state: down; no EC2/EBS volume spend, retained snapshot `snap-0758119609e81ab7f`

## Local Artifacts

- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/suite.json`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/manifest.md`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/aws-suite-ssm-summary.md`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/suite-audit-local-current-head.log`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/suite-dry-run-local-current-head.log`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/suite-dry-run-manifest-current-head.json`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/ssm-sync-suite-invocation.json`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/ssm-build-cli-invocation.json`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/suite-manifest.json`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/results.jsonl`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/results-report.jsonl`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/suite-status.log`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/suite-report.log`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/sidecar-1m-rabitq1-k50-q200-c1.log`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/sidecar-1m-rabitq1-k50-q200-c4-rabitq8-tid-sorted.log`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/cloud-status-after-down.log`

## Artifact Pull / Shutdown

The complete remote artifacts were copied back into the benchmark packet from:

```text
/var/lib/pgsql/build/ecaz/benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/
```

The profile was snapshotted and torn down after artifact retrieval:

```text
snapshot: snap-0758119609e81ab7f
state: down
cost: ~$0.00/hr running, ~$4.00/mo retained storage
```

## Key Result Lines

Baseline recall, q=500:

```text
nprobe=256 recall@10=0.9936 ci95=0.9910-0.9955 ndcg@10=0.9998
```

Baseline latency, q=200:

```text
nprobe=256 p50=69.1 ms p95=75.7 ms p99=80.2 ms max=109.5 ms
```

EXPLAIN:

```text
index_size=298 MB
postings_scored=293022
posting_pages_read=10975
rerank_rows=50
heap_blocks_fetched=48
approximate_scan_elapsed_us=79706
exact_rerank_elapsed_us=944
execution_time=84.427 ms
```

Sidecar real-I/O, q=200, nprobe=128, candidate_k=50:

```text
f16 random-id c1: recall@10=0.9815 sidecar_p50=18.761 ms sidecar_p95=324.069 ms sidecar_p99=529.692 ms total_bound_p50=63.026 ms sidecar_size=2.83 GiB
f16 tid-sorted c1: recall@10=0.9815 sidecar_p50=0.523 ms sidecar_p95=0.787 ms sidecar_p99=1.920 ms total_bound_p50=43.619 ms sidecar_size=2.83 GiB
rabitq8 random-id c1: recall@10=0.9455 sidecar_p50=1.918 ms sidecar_p95=4.819 ms sidecar_p99=11.585 ms total_bound_p50=45.166 ms sidecar_size=1.43 GiB
rabitq8 tid-sorted c1: recall@10=0.9455 sidecar_p50=0.413 ms sidecar_p95=0.437 ms sidecar_p99=0.535 ms total_bound_p50=43.499 ms sidecar_size=1.43 GiB
rabitq8 tid-sorted c4: recall@10=0.9455 sidecar_p50=1.121 ms sidecar_p95=1.723 ms sidecar_p99=334.866 ms total_bound_p50=41.615 ms sidecar_size=1.43 GiB
```
