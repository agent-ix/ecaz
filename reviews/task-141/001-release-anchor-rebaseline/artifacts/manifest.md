# Task 141 Packet 001 Artifact Manifest

- Head SHA: `6926269716f7ed8a846247c4314c588a95707aed`
- Task bucket: `reviews/task-141/001-release-anchor-rebaseline/`
- Timestamp: `2026-07-04`
- Lane: local PG18 SPIRE multi-instance, one coordinator plus three local worker instances on one host.
- Fixture: staged real corpora from `data/staged-current`, `source_identity=include`, `storage_format=rabitq`, `boundary_replica_count=0`, 200 production-read queries.
- Surface isolation: each anchor cell used its own four-instance local PG run and its own coordinator/remote indexes.
- Evidence runner: `ecaz bench suite`; the checked-in config is `task141-release-anchor-suite.json`.
- Top-level output note: `results-r2.jsonl` is empty because `spire-local-multinode` writes measured rows inside each fixture's nested `bench-suite/results.jsonl`.
- Superseded artifact note: `release-50k-n128-b0/` was the first failed attempt and is not cited; all accepted cells use the `*-r2` directories below.

## Commands

Dry run:

```text
target/debug/ecaz bench suite run --config reviews/task-141/001-release-anchor-rebaseline/artifacts/task141-release-anchor-suite.json --manifest-output reviews/task-141/001-release-anchor-rebaseline/artifacts/dry-run-suite-manifest-r2.json --results-output reviews/task-141/001-release-anchor-rebaseline/artifacts/results-r2.jsonl --log-file reviews/task-141/001-release-anchor-rebaseline/artifacts/suite-dry-run-r2.log --dry-run
```

Measurement:

```text
target/debug/ecaz bench suite run --config reviews/task-141/001-release-anchor-rebaseline/artifacts/task141-release-anchor-suite.json --manifest-output reviews/task-141/001-release-anchor-rebaseline/artifacts/suite-manifest-r2.json --results-output reviews/task-141/001-release-anchor-rebaseline/artifacts/results-r2.jsonl --log-file reviews/task-141/001-release-anchor-rebaseline/artifacts/suite-run-r2.log
```

After the backend-profile row stamping change at `692626971`, the nested reports were regenerated from each nested manifest:

```text
target/debug/ecaz bench suite report --manifest <cell>/bench-suite/suite-manifest.json --results-output <cell>/bench-suite/results.jsonl
```

Validation for the code slice:

```text
cargo test -p ecaz-cli release_guard -- --nocapture
cargo test -p ecaz-cli socket_port_discovery_empty_dir_documents_coordinator_fallback -- --nocapture
cargo build -p ecaz-cli
```

All three validation commands passed. `cargo build -p ecaz-cli` emitted one existing warning about `LoadedDistributedPlacementConfig.path` being unread.

## Primary Artifacts

| Artifact | Purpose |
| --- | --- |
| `task141-release-anchor-suite.json` | Four-cell suite config: release 50k n128/b0, release 50k n1024/b0, release 100k n1024/b0, debug 50k n1024/b0. |
| `suite-manifest-r2.json`, `suite-run-r2.log` | Top-level suite manifest and run log. |
| `<cell>/local-multinode.log` | Four-node fixture setup log for each cited cell. |
| `<cell>/bench-suite/suite-manifest.json` | Nested manifest with coordinator plus per-node `backend_nodes[].build_profile`. |
| `<cell>/bench-suite/results.jsonl` | Nested storage, query latency/recall, and production-read profile rows; latency rows include `backend_build_profile` and `backend_node_profiles`. |
| `<cell>/bench-suite/production-read-k10-default.log` | Raw production-read query and profile output. |
| `<cell>/bench-suite/storage.log` | Coordinator storage output. |
| `<cell>/{coordinator-load,remote-load-node-2,remote-load-node-3,remote-load-node-4}.log` | Load and index-build timing evidence. |

Generated shard TSVs under `<cell>/distributed-correctness/node-*` are regenerable corpus data and are intentionally not committed.

## Backend Provenance

Every cited nested `suite-manifest.json` records four backend nodes. Release cells report:

```text
coordinator:39700:release,local-port-39701:39701:release,local-port-39702:39702:release,local-port-39703:39703:release
```

The debug comparator reports:

```text
coordinator:39700:debug,local-port-39701:39701:debug,local-port-39702:39702:debug,local-port-39703:39703:debug
```

The same profile labels are stamped into every cited `spire-pipeline` row in nested `results.jsonl`.

## Anchor Query Latency and Recall

These are the end-to-end production-read query rows from nested `results.jsonl`, not the per-phase profile rows.

| Cell | nprobe | build | query p50 | query p95 | recall@10 |
| --- | ---: | --- | ---: | ---: | ---: |
| 50k n128/b0 | 8 | release | 64.966 ms | 67.693 ms | 0.8920 |
| 50k n128/b0 | 16 | release | 66.694 ms | 69.357 ms | 0.9375 |
| 50k n128/b0 | 32 | release | 69.993 ms | 73.491 ms | 0.9725 |
| 50k n128/b0 | 64 | release | 77.131 ms | 86.048 ms | 0.9865 |
| 50k n128/b0 | 96 | release | 85.229 ms | 90.832 ms | 0.9900 |
| 50k n1024/b0 | 8 | release | 109.010 ms | 116.638 ms | 0.7240 |
| 50k n1024/b0 | 16 | release | 107.727 ms | 113.387 ms | 0.8210 |
| 50k n1024/b0 | 32 | release | 107.254 ms | 111.812 ms | 0.8895 |
| 50k n1024/b0 | 64 | release | 108.127 ms | 116.733 ms | 0.9375 |
| 50k n1024/b0 | 96 | release | 110.109 ms | 121.102 ms | 0.9575 |
| 100k n1024/b0 | 8 | release | 104.843 ms | 109.490 ms | 0.6715 |
| 100k n1024/b0 | 16 | release | 106.654 ms | 111.955 ms | 0.7860 |
| 100k n1024/b0 | 32 | release | 108.299 ms | 114.920 ms | 0.8615 |
| 100k n1024/b0 | 64 | release | 113.852 ms | 119.286 ms | 0.9105 |
| 100k n1024/b0 | 96 | release | 111.314 ms | 116.191 ms | 0.9355 |
| 50k n1024/b0 | 8 | debug | 574.992 ms | 593.179 ms | 0.7240 |
| 50k n1024/b0 | 16 | debug | 579.914 ms | 596.068 ms | 0.8210 |
| 50k n1024/b0 | 32 | debug | 588.847 ms | 605.642 ms | 0.8895 |
| 50k n1024/b0 | 64 | debug | 608.748 ms | 630.598 ms | 0.9375 |
| 50k n1024/b0 | 96 | debug | 630.760 ms | 651.126 ms | 0.9575 |

Recall is unchanged between the debug and release 50k n1024/b0 matched cells.

## Production-Read Profile Rows

These rows are attribution signals for the production-read step. They are not the end-to-end query p50.

| Cell | nprobe | build | total p50 | total p95 | candidate p50 | heap p50 | merge p50 |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 50k n128/b0 | 8 | release | 33.000 ms | 34.000 ms | 4.000 ms | 4.000 ms | 0.000 ms |
| 50k n128/b0 | 16 | release | 34.000 ms | 37.000 ms | 7.000 ms | 7.000 ms | 0.000 ms |
| 50k n128/b0 | 32 | release | 38.000 ms | 41.000 ms | 11.000 ms | 11.000 ms | 0.000 ms |
| 50k n128/b0 | 64 | release | 45.000 ms | 48.000 ms | 21.000 ms | 21.000 ms | 0.000 ms |
| 50k n128/b0 | 96 | release | 52.000 ms | 56.000 ms | 31.000 ms | 31.000 ms | 0.000 ms |
| 50k n1024/b0 | 8 | release | 49.000 ms | 53.000 ms | 3.000 ms | 3.000 ms | 0.000 ms |
| 50k n1024/b0 | 16 | release | 50.000 ms | 52.000 ms | 3.000 ms | 3.000 ms | 0.000 ms |
| 50k n1024/b0 | 32 | release | 49.000 ms | 51.000 ms | 4.000 ms | 4.000 ms | 0.000 ms |
| 50k n1024/b0 | 64 | release | 51.000 ms | 55.000 ms | 6.000 ms | 6.000 ms | 0.000 ms |
| 50k n1024/b0 | 96 | release | 52.000 ms | 57.000 ms | 7.000 ms | 7.000 ms | 0.000 ms |
| 100k n1024/b0 | 8 | release | 48.000 ms | 50.000 ms | 3.000 ms | 3.000 ms | 0.000 ms |
| 100k n1024/b0 | 16 | release | 49.000 ms | 51.000 ms | 5.000 ms | 5.000 ms | 0.000 ms |
| 100k n1024/b0 | 32 | release | 51.000 ms | 54.000 ms | 6.000 ms | 6.000 ms | 0.000 ms |
| 100k n1024/b0 | 64 | release | 55.000 ms | 58.000 ms | 9.000 ms | 9.000 ms | 0.000 ms |
| 100k n1024/b0 | 96 | release | 54.000 ms | 57.000 ms | 11.000 ms | 12.000 ms | 0.000 ms |
| 50k n1024/b0 | 8 | debug | 242.000 ms | 254.000 ms | 21.000 ms | 24.000 ms | 0.000 ms |
| 50k n1024/b0 | 16 | debug | 251.000 ms | 264.000 ms | 30.000 ms | 33.000 ms | 0.000 ms |
| 50k n1024/b0 | 32 | debug | 262.000 ms | 279.000 ms | 46.000 ms | 49.000 ms | 0.000 ms |
| 50k n1024/b0 | 64 | debug | 287.000 ms | 307.000 ms | 76.000 ms | 80.000 ms | 0.000 ms |
| 50k n1024/b0 | 96 | debug | 309.000 ms | 332.000 ms | 105.000 ms | 110.000 ms | 0.000 ms |

## Debug to Release Distortion

Matched cell: 50k n1024/b0.

| nprobe | query p50 debug/release | profile total p50 debug/release | recall delta |
| ---: | ---: | ---: | ---: |
| 8 | 5.27x | 4.94x | 0.0000 |
| 16 | 5.38x | 5.02x | 0.0000 |
| 32 | 5.49x | 5.35x | 0.0000 |
| 64 | 5.63x | 5.63x | 0.0000 |
| 96 | 5.73x | 5.94x | 0.0000 |

Build/load distortion on 50k n1024/b0:

| Node | release total | debug total | total ratio | release index build | debug index build | index ratio |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| coordinator | 288.37 s | 2143.01 s | 7.43x | 148.84 s | 1995.54 s | 13.41x |
| remote 2 | 106.72 s | 834.12 s | 7.82x | 58.59 s | 784.33 s | 13.39x |
| remote 3 | 100.80 s | 846.19 s | 8.39x | 54.07 s | 796.08 s | 14.72x |
| remote 4 | 98.23 s | 823.21 s | 8.38x | 53.03 s | 773.39 s | 14.58x |

## Storage

| Cell | coordinator SPIRE index | index bytes | total relation footprint |
| --- | ---: | ---: | ---: |
| 50k n128/b0 release | 44.8 MiB | 46976205 | 840.5 MiB |
| 50k n1024/b0 release | 54.1 MiB | 56727962 | 849.8 MiB |
| 100k n1024/b0 release | 97.5 MiB | 102236160 | 1.6 GiB |
| 50k n1024/b0 debug | 54.1 MiB | 56727962 | 849.8 MiB |

## 87 ms Reconciliation

Task 123 packet 009 reported 100k n1024/b2 nprobe64 at 87.323 ms p50 / 90.365 ms p95 / recall 1.0000 over 32 queries from `target/debug/ecaz`. That packet predated backend-profile manifest and row stamping, so the result was not self-describing as debug-build evidence.

This packet's matched debug/release A/B shows debug is 5.27x to 5.73x slower than release for 50k n1024/b0 query p50. The new 100k n1024/b0 release anchor at nprobe64 is 113.852 ms p50 / 119.286 ms p95 / recall 0.9105. That places the old 87 ms number in the release-regime order of magnitude, not in the later debug grid's 600 ms regime.

The remaining discrepancy is workload/configuration drift:

- Task 123 packet 009 used b2, `top_k=10`, `query_limit=32`, default `rerank_width=0`, and `id` projection.
- Task 139's phase-1 grid was a 50k debug-build grid with different anchor cells and no durable backend provenance.
- Task 123 packet 017 later reran the local multi-instance 100k n1024/b2 family with 200 queries and found `id-prune-off-default` at 796.862 ms p50 / 914.903 ms p95 / recall 1.0000, explicitly retracting the earlier 32-query optimism.

Conclusion: the 87 ms packet-009 row is a short-run, pre-provenance measurement and should not be used as a debug-grid comparator. The release substrate now anchors n1024/b0 at roughly 108-114 ms p50, while the matched debug substrate anchors the same 50k cell at roughly 575-631 ms p50 with unchanged recall.

## Open Limitation

The production-read profile timers are millisecond-granularity. They are usable for Task 141 debug/release distortion, but Task 142 should add finer timing before claiming single-digit millisecond routing or transport wins.
