# Review Request: AWS IVF/RaBitQ Baseline Confirmation Gate

Please review the Task 51 AWS baseline-confirmation measurement packet:

- benchmark packet: `benchmarks/task51-aws-ivf-rabitq-final-gate/`
- checked-in suite config: `benchmarks/task51-aws-ivf-rabitq-final-gate/suite.json`
- review artifacts: `reviews/task-51/011-aws-ivf-rabitq-final-gate/artifacts/`

## Scope

This is an AWS baseline-confirmation packet for IVF/RaBitQ only. It restores the preserved 100 GB snapshot, verifies the existing built index, runs the checked-in `ecaz bench suite` config, snapshots the post-run state, and tears the stack down.

No vchord or pgvectorscale steps were run.

This packet confirms the preserved 1M Graviton IVF/RaBitQ shape. It is not the final Task 51 round result and does not exercise the later Exp 3/5/7 follow-up paths.

Disabled / not exercised in this suite:

- adaptive nprobe: off
- adaptive score-gap / margin-ratio GUCs: not enabled
- scratch SoA batch decode GUC: off
- sidecar real-I/O: not exercised here; measured separately as a harness path

## Database / Index Verification

Precheck on the restored AWS host:

```text
/usr/bin/pg_config
PostgreSQL 18.3
active
tqvector_bench
real_1m_ivf_rabitq1_rerank_rabitq_idx|ec_ivf|{quant_bits=1,rerank=heap_f32,rerank_width=50,storage_format=rabitq}
990000
10000
```

## Suite Status

```text
[suite:task51-aws-ivf-rabitq-final-gate] completed=5 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Results

Recall, q=500:

| nprobe | recall@10 | CI95 | NDCG@10 |
| ---: | ---: | --- | ---: |
| 64 | 0.9716 | 0.9666-0.9759 | 0.9983 |
| 128 | 0.9864 | 0.9828-0.9893 | 0.9993 |
| 256 | 0.9936 | 0.9910-0.9955 | 0.9998 |

Latency, q=200, concurrency=1:

| nprobe | p50 | p95 | p99 |
| ---: | ---: | ---: | ---: |
| 64 | 18.7 ms | 22.2 ms | 23.3 ms |
| 128 | 34.6 ms | 41.5 ms | 48.0 ms |
| 256 | 66.2 ms | 72.5 ms | 75.7 ms |

Storage:

```text
rows=990000
table=15.4 GiB
indexes=340.4 MiB
total=15.7 GiB
ec_ivf index=298.0 MiB
ec_ivf per row=315.6 B
```

## Snapshot / Teardown

Post-run snapshot:

```text
snapshot: profile=10k-medium id=snap-0e0632400184fadd4
```

The snapshot inventory in `docs/aws-bench-workflow.md` was updated with this ID. Teardown evidence is in the benchmark packet under `artifacts/cloud-down-post-final-gate*`.

## Caveats

- `nprobe=64` recall mean q-time is inflated by cold snapshot / exact-truth setup effects in the first recall sweep. Use the separate latency step for latency claims.
- The suite ran against the cloud-init-installed branch build on AWS. During close-out, `ecaz cloud install` also exposed an AL2023 PG18 path/sudo bug; that fix is covered separately by packet `reviews/task-51/010-cloud-install-al2023-pg18-path/`.
