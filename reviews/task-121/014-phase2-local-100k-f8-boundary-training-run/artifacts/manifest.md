# Artifact Manifest

Task bucket: `reviews/task-121/014-phase2-local-100k-f8-boundary-training-run/`

Head SHA: `96c751d91457499f463a0f17657e018fe17656fe`

Timestamp: 2026-06-23 local time

Lane: local PG18 single-node pipeline evidence. This is not local multi-node evidence and not AWS evidence.

Fixture:

- Database: `tqvector_bench_task121`
- Host/socket: `/home/peter/.pgrx`
- Port: `28818`
- Corpus: `data/staged-current/ec_real_100k_corpus.tsv`
- Queries: `data/staged-current/ec_real_100k_queries.tsv`
- Manifest: `data/staged-current/ec_real_100k_manifest.json`
- Queries used for recall/pipeline: 200
- k: 10

Suite command:

```sh
target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-121/014-phase2-local-100k-f8-boundary-training-run/artifacts/suite-phase2-local-100k-f8-boundary-training-run.json --manifest-output reviews/task-121/014-phase2-local-100k-f8-boundary-training-run/artifacts/suite-phase2-local-100k-f8-boundary-training-run-manifest.json --results-output reviews/task-121/014-phase2-local-100k-f8-boundary-training-run/artifacts/suite-phase2-local-100k-f8-boundary-training-run-results.jsonl --log-file reviews/task-121/014-phase2-local-100k-f8-boundary-training-run/artifacts/suite-phase2-local-100k-f8-boundary-training-run.log
```

Run notes:

- The suite was stopped by terminating the active PostgreSQL backend after `pipeline-100k_b0_tr10_f8` completed.
- Therefore `pipeline-100k_b0_tr50_f8` is recorded as failed with exit code 1, and the remaining two pipelines are pending.
- The suite did not leave a `suite-results.jsonl`; cited pipeline metrics come from `pipeline-100k_b0_tr10_f8.log`.

Artifacts:

| artifact | purpose |
| --- | --- |
| `suite-phase2-local-100k-f8-boundary-training-run.json` | checked-in `ecaz bench suite` config |
| `suite-phase2-local-100k-f8-boundary-training-run-audit.log` | suite audit output |
| `suite-phase2-local-100k-f8-boundary-training-run-manifest.json` | structured suite step status |
| `suite-phase2-local-100k-f8-boundary-training-run.log` | suite runner log |
| `suite-phase2-local-100k-f8-boundary-training-run.script.log` | terminal capture for the run |
| `precheck-host.log` | PG18/ecaz precheck |
| `load-100k_*.log` | load/index build logs for the four 100k f8 cells |
| `storage-100k_*.log` | storage results for the four 100k f8 cells |
| `truth-cache-100k-q200-k10.log` | truth-cache command log |
| `pipeline-100k_b0_tr10_f8.log` | completed baseline pipeline recall/latency and diagnostic summaries |
| `summary-100k-f8-baseline.md` | compact cited summary |

Not committed:

- `truth-cache-100k-q200-k10.json`
- `pipeline-100k_*-funnel.jsonl`
- `pipeline-100k_*-stage-containment.jsonl`
- corpus/query TSVs

Key result lines cited:

- Storage:
  - `b0_tr10_f8`: 79.7 MiB index, 835.8 B/row, 1.6 GiB table total.
  - `b0_tr50_f8`: 79.6 MiB index, 835.2 B/row, 1.6 GiB table total.
  - `b1_tr10_f8`: 157.9 MiB index, 1655.2 B/row, 1.7 GiB table total.
  - `b1_tr50_f8`: 157.8 MiB index, 1654.5 B/row, 1.7 GiB table total.
- Completed baseline pipeline `b0_tr10_f8`:
  - nprobe 4: p50 125.369 ms, p95 163.540 ms, recall@10 0.5500.
  - nprobe 8: p50 245.514 ms, p95 307.142 ms, recall@10 0.7250.
  - nprobe 12: p50 386.604 ms, p95 459.467 ms, recall@10 0.8010.
  - nprobe 16: p50 518.468 ms, p95 608.491 ms, recall@10 0.8525.
  - nprobe 24: p50 817.531 ms, p95 916.031 ms, recall@10 0.9045.
  - nprobe 32: p50 1104.833 ms, p95 1265.117 ms, recall@10 0.9310.
  - nprobe 48: p50 1715.345 ms, p95 1914.705 ms, recall@10 0.9645.
  - nprobe 64: p50 2308.951 ms, p95 2485.750 ms, recall@10 0.9825.
  - nprobe 96: p50 3455.559 ms, p95 3687.544 ms, recall@10 0.9975.
