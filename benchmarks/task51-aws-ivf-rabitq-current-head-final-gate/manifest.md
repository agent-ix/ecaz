# Benchmark Manifest: AWS Current-HEAD IVF/RaBitQ Final Gate

- head SHA at suite config commit: `902e8e066944d4cabfb26ee5cc9039b466856891`
- AWS host head SHA: `902e8e066944d4cabfb26ee5cc9039b466856891`
- task bucket: `reviews/task-51/`
- review packet: `reviews/task-51/017-aws-current-head-final-gate/`
- benchmark packet: `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/`
- lane: AWS current-head final gate, IVF and RaBitQ only
- fixture: preserved real DBpedia corpus, 990000 corpus rows, 10000 query rows
- restored snapshot: `snap-0e0632400184fadd4`
- instance shape: DB `m8g.2xlarge`, loader `c8g.medium`
- storage format: `rabitq`
- preserved index reloptions: `{quant_bits=1,rerank=heap_f32,rerank_width=50,storage_format=rabitq}`
- rerank mode: preserved in-index `heap_f32`, `rerank_width=50`; sidecar harness measured separately with `candidate_k=50`
- isolated one-index-per-table surface: yes for preserved AWS table/index
- vchord / pgvectorscale: not run
- suite runner: `ecaz bench suite`
- suite command id: `70df8076-1c85-4481-b1c9-a3e8bdbd7f88`
- suite status: `Success`, response code `0`, elapsed `PT31M14.927S`
- post-run stack state: paused; no instance spend, remote artifacts retained

## Suite Config

- `suite.json`

The suite includes:

- preserved DB/index precheck
- q=500 recall sanity at `nprobe=256`
- q=200 latency sanity at `nprobe=256`
- EXPLAIN counters at `nprobe=256`
- sidecar c1 full scoped matrix for `f16` and `rabitq8` across `random-id` and `tid-sorted`
- sidecar c4 follow-up for `rabitq8` / `tid-sorted`

The sidecar steps use `allow_unsafe_index_shape=true` because the preserved AWS
snapshot contains a `rerank=heap_f32` index, not a `rerank=off` sidecar-only
index. Treat these sidecar values as real sidecar I/O measurements on the
preserved candidate frontier, not as a product in-index sidecar storage result.

## Local Artifacts

- `artifacts/suite-audit-local-current-head.log`
- `artifacts/suite-dry-run-local-current-head.log`
- `artifacts/suite-dry-run-manifest-current-head.json`
- `artifacts/cloud-install-current-branch.log`
- `artifacts/cloud-install-current-branch-retry.log`
- `artifacts/cargo-build-release-ecaz-cli.log`
- `artifacts/ssm-sync-suite-commands.json`
- `artifacts/ssm-sync-suite-invocation.json`
- `artifacts/ssm-build-cli-commands.json`
- `artifacts/ssm-build-cli-invocation.json`
- `artifacts/ssm-run-current-head-suite-commands.json`
- `artifacts/aws-suite-ssm-summary.md`

Several Terraform/status setup logs are also present in `artifacts/` and record
the restored-snapshot path, vCPU-limit retry, and final state sync.

## Remote Artifacts

The AWS run wrote the complete suite artifacts on the DB host:

```text
/var/lib/pgsql/build/ecaz/benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/
```

Expected remote files include:

- `suite-run.log`
- `suite-status.log`
- `suite-report.log`
- `suite-manifest.json`
- `results.jsonl`
- `results-report.jsonl`
- `precheck-preserved-1m-ivf-rabitq.log`
- `recall-1m-rabitq1-rerank-q500-p256.log`
- `latency-1m-rabitq1-rerank-p256.log`
- `explain-1m-rabitq1-rerank-p256.log`
- `sidecar-1m-rabitq1-k50-q200-c1.log`
- `sidecar-1m-rabitq1-k50-q200-c4-rabitq8-tid-sorted.log`

These files are not yet copied locally because non-escalated SSM artifact sync
failed with an endpoint error. The stack is paused to preserve them without
running instance spend.

## Key Result Lines

```text
suite_status=Success
response_code=0
elapsed=PT31M14.927S
```

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
