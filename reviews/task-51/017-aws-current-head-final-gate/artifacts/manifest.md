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
- post-run stack state: paused; no instance spend, remote artifacts retained

## Local Artifacts

- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/suite.json`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/manifest.md`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/aws-suite-ssm-summary.md`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/suite-audit-local-current-head.log`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/suite-dry-run-local-current-head.log`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/suite-dry-run-manifest-current-head.json`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/ssm-sync-suite-invocation.json`
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/ssm-build-cli-invocation.json`

## Remote Artifacts

The complete remote artifacts remain on the paused DB host at:

```text
/var/lib/pgsql/build/ecaz/benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/
```

They include `results.jsonl`, `results-report.jsonl`, `suite-manifest.json`,
per-step logs, and the complete sidecar rows. A non-escalated SSM artifact-sync
attempt failed with an endpoint error, and no approval request was made.

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

Sidecar row visible before SSM truncation:

```text
variant=f16 read_mode=random-id concurrency=1 nprobe=128 candidate_k=50 recall@10=0.9815 sidecar_io_p50=18.707 ms sidecar_p50=18.761 ms sidecar_io_p95=324.014 ms sidecar_p95=324.069 ms sidecar_size=2.83 GiB
```
