# Task 51 Local IVF/RaBitQ 990k Current-Shape Suite

- head SHA: `d72246e6cad5bab99e0889798fd75247978346a7`
- timestamp: `2026-05-23T08:01:32Z`
- benchmark packet: `benchmarks/task51-local-ivf-rabitq-990k/`
- runner: `ecaz bench suite`
- SuiteConfig: `benchmarks/task51-local-ivf-rabitq-990k/suite.json`
- suite manifest: `benchmarks/task51-local-ivf-rabitq-990k/artifacts/suite-manifest.json`
- results: `benchmarks/task51-local-ivf-rabitq-990k/artifacts/results.jsonl`
- lane: local PG18 / WSL2 only
- AWS: not used
- competitors: none; this packet is IVF/RaBitQ only
- table surface: isolated one-index-per-table prefix `task51_local_990k_ivf_rabitq1_n1024_w50`
- corpus: staged anchor corpus, 990000 rows, 10000 query rows, dim 1536
- profile: `ec_ivf`
- storage format: `rabitq`
- reloptions: `nlists=1024`, `nprobe=256`, `training_sample_rows=10000`, `quant_bits=1`, `rerank=heap_f32`, `rerank_width=50`, `storage_format=rabitq`
- rerank mode: heap f32 rerank width 50
- recall query limit: 100 local cost waiver
- latency iterations: 100, concurrency 1

## Commands

Audit:

```text
target/release/ecaz --log-file benchmarks/task51-local-ivf-rabitq-990k/artifacts/suite-audit.log bench suite audit --config benchmarks/task51-local-ivf-rabitq-990k/suite.json
```

Dry run:

```text
target/release/ecaz --log-file benchmarks/task51-local-ivf-rabitq-990k/artifacts/suite-dry-run.log bench suite run --dry-run --config benchmarks/task51-local-ivf-rabitq-990k/suite.json --manifest-output benchmarks/task51-local-ivf-rabitq-990k/artifacts/suite-manifest.json
```

Run:

```text
target/release/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-ivf-rabitq-990k/artifacts/suite-run.log bench suite run --config benchmarks/task51-local-ivf-rabitq-990k/suite.json --manifest-output benchmarks/task51-local-ivf-rabitq-990k/artifacts/suite-manifest.json
```

Status:

```text
target/release/ecaz --log-file benchmarks/task51-local-ivf-rabitq-990k/artifacts/suite-status.log bench suite status --manifest benchmarks/task51-local-ivf-rabitq-990k/artifacts/suite-manifest.json
```

Report:

```text
target/release/ecaz --log-file benchmarks/task51-local-ivf-rabitq-990k/artifacts/suite-report.log bench suite report --manifest benchmarks/task51-local-ivf-rabitq-990k/artifacts/suite-manifest.json --results-output benchmarks/task51-local-ivf-rabitq-990k/artifacts/results.jsonl
```

## Status

`suite-status.log`:

```text
[suite:task51-local-ivf-rabitq-990k] completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Key Results

Load:

- copy corpus: `461.96s`
- encode corpus: `169.83s`
- copy queries: `4.52s`
- build index: `1671.85s`
- total load: `2420.91s`

Recall, q=100, k=10:

| nprobe | recall@10 | NDCG@10 | mean q-time |
| ---: | ---: | ---: | ---: |
| 64 | 0.9570 | 0.9971 | 320.75 ms |
| 96 | 0.9690 | 0.9977 | 419.90 ms |
| 128 | 0.9750 | 0.9986 | 561.32 ms |
| 192 | 0.9820 | 0.9991 | 822.94 ms |
| 256 | 0.9850 | 0.9995 | 1103.46 ms |

Latency, q=100, concurrency 1:

| nprobe | p50 | p95 | p99 | mean |
| ---: | ---: | ---: | ---: | ---: |
| 64 | 285.2 ms | 351.5 ms | 371.7 ms | 288.2 ms |
| 96 | 429.8 ms | 515.9 ms | 554.7 ms | 434.6 ms |
| 128 | 566.0 ms | 659.8 ms | 687.9 ms | 566.6 ms |
| 192 | 806.2 ms | 905.8 ms | 950.7 ms | 809.2 ms |
| 256 | 1083.8 ms | 1197.9 ms | 1277.8 ms | 1089.3 ms |

Storage:

- rows: `990000`
- table: `15.4 GiB`
- total: `15.7 GiB`
- ec_ivf RaBitQ index: `298.3 MiB`
- ec_ivf index per row: `316.0 B`

Explain counters:

- p128 selected lists: `128`
- p128 candidates scored: `138476`
- p128 rerank rows: `50`
- p128 approximate scan: `600582 us`
- p128 exact rerank: `4597 us`
- p256 selected lists: `256`
- p256 candidates scored: `287124`
- p256 rerank rows: `50`
- p256 approximate scan: `1219998 us`
- p256 exact rerank: `2010 us`

## Caveats

- This is local PG18/WSL2 evidence only, not Graviton evidence.
- The recall query count is q=100 as a local cost waiver; final AWS gate should use the agreed larger query count if spend and time allow.
- The suite intentionally did not run vchord or pgvectorscale.
- The staged corpus manifest has prefix `ec_real_ann_benchmarks_anchor`; the suite used `--allow-manifest-mismatch` because this Task 51 packet creates an isolated current-shape prefix from that staged source.
- `quant_bits` is passed through by the loader with a known profile-registry warning; storage and explain artifacts confirm the index reloptions include `quant_bits=1`.
- `*-debug-aborted.*` artifacts record an aborted debug-binary preflight before database mutation; the successful suite evidence is from `target/release/ecaz`.

## Artifacts

- `artifacts/suite-audit.log`: audit output.
- `artifacts/suite-dry-run.log`: dry-run command expansion.
- `artifacts/suite-run.log`: suite run log.
- `artifacts/suite-status.log`: final status.
- `artifacts/suite-report.log`: parsed report.
- `artifacts/suite-manifest.json`: structured suite manifest.
- `artifacts/results.jsonl`: parsed structured results.
- `artifacts/load-990k-rabitq1-n1024-w50.log`: load and index build log.
- `artifacts/recall-990k-rabitq1-n1024-w50.log`: recall table.
- `artifacts/latency-990k-rabitq1-n1024-w50.log`: latency table.
- `artifacts/storage-990k-rabitq1-n1024-w50.log`: storage table.
- `artifacts/explain-990k-rabitq1-n1024-w50-p128.sql`: explain SQL for nprobe 128.
- `artifacts/explain-990k-rabitq1-n1024-w50-p128.log`: explain output for nprobe 128.
- `artifacts/explain-990k-rabitq1-n1024-w50-p256.sql`: explain SQL for nprobe 256.
- `artifacts/explain-990k-rabitq1-n1024-w50-p256.log`: explain output for nprobe 256.
- `artifacts/truth-ec-real-990k-q100-k10.json`: exact truth cache for q=100/k=10.
