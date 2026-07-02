# Task 131 Packet 008 Artifact Manifest

- head SHA: `c0d3f33d04f668ce107f0c738d786252ed720b9a`
- task bucket: `reviews/task-131/`
- packet path: `reviews/task-131/008-phase1-50k-n1024-b2-default-ab/`
- timestamp: `2026-07-01T03:58:34-07:00`
- lane: local multi-instance PG18
- fixture: `ec_real_50k`
- storage format: `rabitq`
- rerank mode: default
- index shape: `n1024 / b2 / nprobe64`
- isolated one-index-per-table surfaces: yes, local multinode fixture with coordinator index `t131_p4_mi_50k_n1024_b2_coord_idx` and remote index `t131_p4_mi_50k_n1024_b2_remote_idx`

## Command

```sh
target/debug/ecaz bench suite run \
  --config reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/task131-phase1-local-mi-ab-suite.json \
  --only mi-50k-n1024-b2-global-preheap-ab \
  --manifest-output reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/50k-n1024-b2-suite-manifest.json \
  --results-output reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/50k-n1024-b2-results.jsonl \
  --log-file reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/50k-n1024-b2-suite-run.log
```

The task-local source run under packet 004 was copied here after pruning generated
distributed-correctness TSV corpus and assignment files. This packet contains no
`*.tsv` or `*.tsv.gz` files.

## Artifacts

- `50k-n1024-b2-suite-manifest.json`: top-level suite manifest.
- `50k-n1024-b2-suite-run.log`: top-level suite run log.
- `50k-n1024-b2-results.jsonl`: top-level suite result stream; the nested command wrote detailed rows under `50k-n1024-b2/bench-suite/results.jsonl`.
- `50k-n1024-b2/`: pruned local multinode run artifacts.
- `50k-n1024-b2/bench-suite/local-real-production-read-suite.json`: nested production-read suite config.
- `50k-n1024-b2/bench-suite/suite-manifest.json`: nested suite manifest.
- `50k-n1024-b2/bench-suite/results.jsonl`: structured storage, recall, latency, and production-read profile rows.
- `50k-n1024-b2/bench-suite/production-read-k10-baseline-default.log`: baseline production-read log.
- `50k-n1024-b2/bench-suite/production-read-k10-global-preheap-on-default.log`: global-preheap production-read log.
- `50k-n1024-b2/bench-suite/storage.log`: coordinator storage log.
- `50k-n1024-b2/local-multinode.log`: local multinode harness log.
- `50k-n1024-b2/coordinator-load.log`, `remote-load-node-*.log`: load and index build logs.

## Key Result Lines

From `50k-n1024-b2/bench-suite/results.jsonl`:

- storage: rows `50000`, table `793.8 MiB`, indexes `130.1 MiB`, total `923.9 MiB`, per-row total `19376.4 B`.
- baseline query metrics: `queries=200`, `nprobe=64`, recall@10 `0.9980`, latency p50/p95/p99 `663.809 / 795.704 / 904.363 ms`.
- global-preheap query metrics: `queries=200`, `nprobe=64`, recall@10 `0.9980`, latency p50/p95/p99 `663.340 / 718.746 / 859.830 ms`.
- baseline production-read profile: `status=ready`, total p50/p95/p99 `393 / 447 / 549 ms`, heap p50/p95/p99 `155 / 197 / 217 ms` on node 2, `160 / 198 / 248 ms` on node 3, `153 / 197 / 255 ms` on node 4, remote heap candidates `6000`, payload rows `6000`, global pre-heap input/candidate/pruned `6000 / 2000 / 4000`, strict/timeout/cancel/degraded skip all `0`.
- global-preheap production-read profile: `status=ready`, total p50/p95/p99 `317 / 345 / 411 ms`, heap p50/p95/p99 `155 / 212 / 233 ms` on node 2, `160 / 198 / 223 ms` on node 3, `153 / 212 / 236 ms` on node 4, remote heap candidates `2000`, payload rows `2000`, global pre-heap input/candidate/pruned `6000 / 2000 / 4000`, strict/timeout/cancel/degraded skip all `0`.
- load/index timing: coordinator index build `2872.58s`; remote index builds `2061.87s`, `1999.13s`, `2016.32s`.

## Notes

- Heap rows avoided: `6000 -> 2000` per 200-query profile, a reduction of `4000` rows.
- Payload bytes avoided are not measurable in this no-payload profile lane; `payload_bytes_sum=0` for both variants because `--production-read-timeline-no-payload` was used to avoid tuple payload materialization noise.
- Per-node timeline rows still report `payload_rows_sum=2000` per node in the global-preheap path, while the aggregate production-read profile row reports `payload_rows_sum=2000` total. Use the aggregate profile row for heap-row A/B comparison.
