# Task 145 Packet 006 Artifact Manifest

- Head SHA: `088054027e0b683fdd47a80fdb9410e1c2f361d9`
- Branch: `task-145-spire-rerank-economy-low-probe`
- Task bucket: `reviews/task-145/006-remote-rerank-width-ab-rerun`
- Generated: 2026-07-06
- Runner: `target/release/ecaz bench suite run`
- Suite config: `artifacts/task145-remote-rerank-width-ab-rerun-suite.json`
- Suite manifest: `artifacts/suite-manifest.json`
- Top-level suite results: `artifacts/suite-results.jsonl` is empty because each
  `spire-local-multinode` step writes nested `bench-suite/results.jsonl`.

## Scope

This packet reruns the Task 145 remote rerank-width A/B after packet 005's
remote heap frontier fix, plus the follow-up correction that keeps the automatic
width-0 production top-k path bounded to `top_k`.

Each cell uses isolated `spire-local-multinode` PG18 surfaces with release
`ecaz.so` on the coordinator and all three remotes. `local-multinode.log`
records:

- `install_profile=release`
- `node_build_profile ... profile=release` for node 1/coordinator and nodes
  2-4/remotes
- `bench-suite/results.jsonl` also records
  `backend_build_profile=release` and `backend_node_profiles=...:release`

The A/B variants are:

- `rerank-full`: `ec_spire.rerank_width=0`,
  `ec_spire.max_candidate_rows=100`
- `rerank-50`: `ec_spire.rerank_width=50`,
  `ec_spire.max_candidate_rows=100`

Common held settings:

- `storage_format=rabitq`
- `source_identity=include`
- `top_k=10`
- `queries_limit=200`
- `nprobe=8,16,32,64,96`
- `ec_spire.leaf_score_only_routing=on`
- `ec_spire.route_overfetch_multiplier=1.0`
- `ec_spire.probe_distance_ratio=0`
- `ec_spire.max_remote_payload_bytes_per_row=16384`

## Commands

```bash
target/release/ecaz bench suite run \
  --dry-run \
  --config reviews/task-145/006-remote-rerank-width-ab-rerun/artifacts/task145-remote-rerank-width-ab-rerun-suite.json \
  --manifest-output reviews/task-145/006-remote-rerank-width-ab-rerun/artifacts/suite-manifest-dry-run-r4.json \
  --log-file reviews/task-145/006-remote-rerank-width-ab-rerun/artifacts/suite-dry-run-r4.log

target/release/ecaz bench suite audit \
  --config reviews/task-145/006-remote-rerank-width-ab-rerun/artifacts/task145-remote-rerank-width-ab-rerun-suite.json \
  --log-file reviews/task-145/006-remote-rerank-width-ab-rerun/artifacts/suite-audit-r4.log

target/release/ecaz bench suite run \
  --config reviews/task-145/006-remote-rerank-width-ab-rerun/artifacts/task145-remote-rerank-width-ab-rerun-suite.json \
  --manifest-output reviews/task-145/006-remote-rerank-width-ab-rerun/artifacts/suite-manifest.json \
  --results-output reviews/task-145/006-remote-rerank-width-ab-rerun/artifacts/suite-results.jsonl \
  --log-file reviews/task-145/006-remote-rerank-width-ab-rerun/artifacts/suite-run.log

script -q -c "cargo test production_scan_heap_frontier --no-default-features --features pg18" \
  reviews/task-145/006-remote-rerank-width-ab-rerun/artifacts/cargo-test-production-scan-heap-frontier.log
```

## Key Results

At nprobe 96:

| Cell | Variant | distinct_recall@k | latency_p50 | latency_p95 | remote_heap_candidate_sum | global_pre_heap_candidate_sum |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 10k n128 | rerank-full | 1.0000 | 219.866 ms | 224.686 ms | 60000 | 20000 |
| 10k n128 | rerank-50 | 1.0000 | 132.399 ms | 137.166 ms | 30000 | 10000 |
| 50k n1024 | rerank-full | 0.9595 | 222.755 ms | 229.324 ms | 60000 | 20000 |
| 50k n1024 | rerank-50 | 0.9595 | 140.584 ms | 144.977 ms | 30000 | 10000 |
| 100k n1024 | rerank-full | 0.9570 | 227.666 ms | 234.037 ms | 60000 | 20000 |
| 100k n1024 | rerank-50 | 0.9570 | 141.295 ms | 147.078 ms | 30000 | 10000 |

Full vs width-50 identity JSONL files are byte-identical for all three cells:

- 10k n128: `cmp` exit 0, 1000 lines per identity file
- 50k n1024: `cmp` exit 0, 1000 lines per identity file
- 100k n1024: `cmp` exit 0, 1000 lines per identity file

All nprobe recall and latency rows:

| Cell | Variant | nprobe | distinct_recall@k | latency_p50 | latency_p95 |
| --- | --- | ---: | ---: | ---: | ---: |
| 10k n128 | rerank-full | 8 | 0.9935 | 207.434 ms | 220.454 ms |
| 10k n128 | rerank-full | 16 | 0.9970 | 216.749 ms | 223.553 ms |
| 10k n128 | rerank-full | 32 | 1.0000 | 221.839 ms | 230.124 ms |
| 10k n128 | rerank-full | 64 | 1.0000 | 219.200 ms | 225.548 ms |
| 10k n128 | rerank-full | 96 | 1.0000 | 219.866 ms | 224.686 ms |
| 10k n128 | rerank-50 | 8 | 0.9935 | 128.085 ms | 133.636 ms |
| 10k n128 | rerank-50 | 16 | 0.9970 | 132.317 ms | 137.215 ms |
| 10k n128 | rerank-50 | 32 | 1.0000 | 130.102 ms | 135.618 ms |
| 10k n128 | rerank-50 | 64 | 1.0000 | 131.376 ms | 135.292 ms |
| 10k n128 | rerank-50 | 96 | 1.0000 | 132.399 ms | 137.166 ms |
| 50k n1024 | rerank-full | 8 | 0.7590 | 213.153 ms | 221.549 ms |
| 50k n1024 | rerank-full | 16 | 0.8490 | 220.558 ms | 227.516 ms |
| 50k n1024 | rerank-full | 32 | 0.9105 | 223.991 ms | 230.498 ms |
| 50k n1024 | rerank-full | 64 | 0.9475 | 221.096 ms | 226.429 ms |
| 50k n1024 | rerank-full | 96 | 0.9595 | 222.755 ms | 229.324 ms |
| 50k n1024 | rerank-50 | 8 | 0.7590 | 136.576 ms | 141.123 ms |
| 50k n1024 | rerank-50 | 16 | 0.8490 | 133.982 ms | 139.108 ms |
| 50k n1024 | rerank-50 | 32 | 0.9105 | 134.538 ms | 138.405 ms |
| 50k n1024 | rerank-50 | 64 | 0.9475 | 135.640 ms | 139.277 ms |
| 50k n1024 | rerank-50 | 96 | 0.9595 | 140.584 ms | 144.977 ms |
| 100k n1024 | rerank-full | 8 | 0.7155 | 221.100 ms | 232.131 ms |
| 100k n1024 | rerank-full | 16 | 0.8270 | 223.714 ms | 231.362 ms |
| 100k n1024 | rerank-full | 32 | 0.8895 | 226.281 ms | 233.966 ms |
| 100k n1024 | rerank-full | 64 | 0.9375 | 226.813 ms | 234.043 ms |
| 100k n1024 | rerank-full | 96 | 0.9570 | 227.666 ms | 234.037 ms |
| 100k n1024 | rerank-50 | 8 | 0.7155 | 135.725 ms | 141.237 ms |
| 100k n1024 | rerank-50 | 16 | 0.8270 | 137.528 ms | 143.130 ms |
| 100k n1024 | rerank-50 | 32 | 0.8895 | 136.594 ms | 140.314 ms |
| 100k n1024 | rerank-50 | 64 | 0.9375 | 138.258 ms | 142.490 ms |
| 100k n1024 | rerank-50 | 96 | 0.9570 | 141.295 ms | 147.078 ms |

Storage evidence:

| Cell | Index | Size | Per row |
| --- | --- | ---: | ---: |
| 10k n128 | `t145_r6_10_n128_coord_idx` | 10.1 MiB | 1060.0 B |
| 50k n1024 | `t145_r6_50_n1024_coord_idx` | 54.5 MiB | 1142.0 B |
| 100k n1024 | `t145_r6_100_n1024_coord_idx` | 97.9 MiB | 1026.1 B |

Focused validation:

- `cargo test production_scan_heap_frontier --no-default-features --features pg18`
- Result: 2 passed, 0 failed

## Artifact Inventory

Top-level packet artifacts:

- `artifacts/task145-remote-rerank-width-ab-rerun-suite.json`
- `artifacts/suite-manifest.json`
- `artifacts/suite-run.log`
- `artifacts/suite-results.jsonl` (empty top-level results; nested results are
  authoritative for these multinode steps)
- `artifacts/suite-manifest-dry-run-r4.json`
- `artifacts/suite-dry-run-r4.log`
- `artifacts/suite-audit-r4.log`
- `artifacts/cargo-test-production-scan-heap-frontier.log`

Per-cell committed artifacts:

- `artifacts/remote-10k-n128-r6/local-multinode.log`
- `artifacts/remote-10k-n128-r6/bench-suite/local-real-production-read-suite.json`
- `artifacts/remote-10k-n128-r6/bench-suite/suite-manifest.json`
- `artifacts/remote-10k-n128-r6/bench-suite/suite-run.log`
- `artifacts/remote-10k-n128-r6/bench-suite/results.jsonl`
- `artifacts/remote-10k-n128-r6/bench-suite/storage.log`
- `artifacts/remote-10k-n128-r6/bench-suite/production-read-k10-rerank-full-default.log`
- `artifacts/remote-10k-n128-r6/bench-suite/production-read-k10-rerank-50-default.log`
- `artifacts/remote-10k-n128-r6/bench-suite/production-read-k10-rerank-full-default-identity.jsonl`
- `artifacts/remote-10k-n128-r6/bench-suite/production-read-k10-rerank-50-default-identity.jsonl`
- Same file set under `artifacts/remote-50k-n1024-r6/`
- Same file set under `artifacts/remote-100k-n1024-r6/`

Operational logs, server logs, corpus TSVs, tunnel state, and distributed
correctness raw data are not committed as packet evidence.
