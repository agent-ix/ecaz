# Artifact Manifest: AWS IVF/RaBitQ Baseline Confirmation Gate

- head SHA at suite config commit: `697b6d690b4311f36e95ff6c63f368acda1367b8`
- task bucket: `reviews/task-51/`
- packet path: `reviews/task-51/011-aws-ivf-rabitq-final-gate/`
- benchmark packet: `benchmarks/task51-aws-ivf-rabitq-final-gate/`
- lane: AWS baseline confirmation gate, IVF and RaBitQ only
- role in round: confirms the preserved 1M Graviton IVF/RaBitQ shape before later Exp 3/5/7 follow-ups; not the final Task 51 round result
- fixture: real DBpedia 990000 corpus rows, q=500 recall, q=200 latency
- storage format: `rabitq`
- rerank mode: `heap_f32`, `rerank_width=50`
- isolated one-index-per-table surface: yes
- vchord / pgvectorscale: not run
- adaptive nprobe: off
- adaptive score-gap / margin-ratio GUCs: not enabled
- scratch SoA GUC: off
- sidecar real-I/O: not exercised in this suite

## Artifacts

- `suite-status.log`: AWS suite status, copied from benchmark packet.
- `suite-report.log`: parsed AWS suite report, copied from benchmark packet.
- `storage-1m-rabitq1-rerank.log`: storage accounting, copied from benchmark packet.
- `ssm-preserved-db-precheck-invocation.json`: preserved DB/index verification.
- `cloud-snapshot-post-final-gate-capture.log`: post-run snapshot evidence.

## Key Result Lines

```text
[suite:task51-aws-ivf-rabitq-final-gate] completed=5 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Recall:

```text
nprobe=64 recall@k=0.9716 ndcg@k=0.9983
nprobe=128 recall@k=0.9864 ndcg@k=0.9993
nprobe=256 recall@k=0.9936 ndcg@k=0.9998
```

Latency:

```text
nprobe=64 p50=18.7 ms p95=22.2 ms p99=23.3 ms
nprobe=128 p50=34.6 ms p95=41.5 ms p99=48.0 ms
nprobe=256 p50=66.2 ms p95=72.5 ms p99=75.7 ms
```

Storage:

```text
rows=990000
ec_ivf index=298.0 MiB
ec_ivf per row=315.6 B
reloptions={quant_bits=1,rerank=heap_f32,rerank_width=50,storage_format=rabitq}
```
