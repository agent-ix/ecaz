# Task 131 Packet 007 Artifact Manifest

- head SHA: `56f33f7e34d3d052f6ada3e32aa41ef6a510ab98`
- task bucket: `reviews/task-131/`
- packet path: `reviews/task-131/007-phase1-50k-n128-b4-default-ab/`
- timestamp: `2026-07-01T01:10:04-07:00`
- lane: local multi-instance PG18
- fixture: `ec_real_50k`
- storage format: `rabitq`
- rerank mode: default
- index shape: `n128 / b4 / nprobe96`
- isolated one-index-per-table surfaces: yes, local multinode fixture with coordinator index `t131_p4_mi_50k_n128_b4_coord_idx` and remote index `t131_p4_mi_50k_n128_b4_remote_idx`

## Command

```sh
target/debug/ecaz bench suite run \
  --config reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/task131-phase1-local-mi-ab-suite.json \
  --only mi-50k-n128-b4-global-preheap-ab \
  --manifest-output reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/50k-n128-b4-suite-manifest.json \
  --results-output reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/50k-n128-b4-results.jsonl \
  --log-file reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/50k-n128-b4-suite-run.log
```

The task-local source run under packet 004 was copied here after pruning generated
distributed-correctness TSV corpus and assignment files. This packet contains no
`*.tsv` or `*.tsv.gz` files.

## Artifacts

- `50k-n128-b4-suite-manifest.json`: top-level suite manifest.
- `50k-n128-b4-suite-run.log`: top-level suite run log.
- `50k-n128-b4-results.jsonl`: top-level suite result stream; the nested command wrote detailed rows under `50k-n128-b4/bench-suite/results.jsonl`.
- `50k-n128-b4/`: pruned local multinode run artifacts.
- `50k-n128-b4/bench-suite/local-real-production-read-suite.json`: nested production-read suite config.
- `50k-n128-b4/bench-suite/suite-manifest.json`: nested suite manifest.
- `50k-n128-b4/bench-suite/results.jsonl`: structured storage, recall, latency, and production-read profile rows.
- `50k-n128-b4/bench-suite/production-read-k10-baseline-default.log`: baseline production-read log.
- `50k-n128-b4/bench-suite/production-read-k10-global-preheap-on-default.log`: global-preheap production-read log.
- `50k-n128-b4/bench-suite/storage.log`: coordinator storage log.
- `50k-n128-b4/local-multinode.log`: local multinode harness log.
- `50k-n128-b4/coordinator-load.log`, `remote-load-node-*.log`: load and index build logs.

## Key Result Lines

From `50k-n128-b4/bench-suite/results.jsonl`:

- storage: rows `50000`, table `793.8 MiB`, indexes `198.0 MiB`, total `991.9 MiB`, `t131_p4_mi_50k_n128_b4_coord_idx` `196.9 MiB`.
- baseline query metrics: `queries=200`, `nprobe=96`, recall@10 `1.0000`, latency p50/p95/p99 `2582.977 / 2953.783 / 3514.706 ms`.
- global-preheap query metrics: `queries=200`, `nprobe=96`, recall@10 `1.0000`, latency p50/p95/p99 `2593.483 / 2981.787 / 3492.307 ms`.
- baseline production-read profile: `status=ready`, total p50/p95/p99 `2537 / 3261 / 3522 ms`, heap p50/p95/p99 `3337 / 4157 / 4880 ms`, remote heap candidates `6000`, payload rows `6000`, global pre-heap input/candidate/pruned `6000 / 2000 / 4000`, strict/timeout/cancel/degraded skip all `0`.
- global-preheap production-read profile: `status=ready`, total p50/p95/p99 `1303 / 1758 / 1904 ms`, heap p50/p95/p99 `7 / 11 / 12 ms`, remote heap candidates `2000`, payload rows `2000`, global pre-heap input/candidate/pruned `6000 / 2000 / 4000`, strict/timeout/cancel/degraded skip all `0`.

## Notes

- Heap rows avoided: `6000 -> 2000` per 200-query profile, a reduction of `4000` rows.
- Payload bytes avoided are not measurable in this no-payload profile lane; `payload_bytes_sum=0` for both variants because `--production-read-timeline-no-payload` was used to avoid tuple payload materialization noise.
- Per-node timeline rows still report `payload_rows_sum=2000` per node in the global-preheap path, while the aggregate production-read profile row reports `payload_rows_sum=2000` total. Use the aggregate profile row for heap-row A/B comparison.
