# Task 142 Packet 017 Artifact Manifest

- head SHA: `755f50656dff4de94cec64227b12363a4cda995a`
- task bucket: `reviews/task-142/`
- packet path: `reviews/task-142/017-release-n2048-extension/`
- timestamp: `2026-07-06`
- slice: `nlists=2048` release extension plus epoch-invalidation regression
- lane / fixture / storage / rerank: local PG18 SPIRE multi-instance, one coordinator plus three local worker instances, staged real corpora, `source_identity=include`, `storage_format=rabitq`, `boundary_replica_count=0`, `rerank_width=0`
- isolated one-index-per-table vs shared-table surface: isolated local multinode fixture with one coordinator index and one remote index per remote node; not a shared-table run
- evidence runner: `ecaz bench suite`

## Commands

Release CLI build:

```text
cargo build --release -p ecaz-cli
```

Focused regression test:

```text
script -q -c "cargo test --lib collect_cached_resolved_scan_plan_selection_reloads_on_epoch_change" reviews/task-142/017-release-n2048-extension/artifacts/epoch-invalidation-test.log
```

Dry run:

```text
target/release/ecaz --database postgres --host /tmp --port 28818 bench suite run --config reviews/task-142/017-release-n2048-extension/artifacts/task142-release-n2048-extension-suite.json --dry-run --artifact-dir reviews/task-142/017-release-n2048-extension/artifacts --manifest-output reviews/task-142/017-release-n2048-extension/artifacts/suite-manifest-dry-run.json --results-output reviews/task-142/017-release-n2048-extension/artifacts/suite-results-dry-run.jsonl
```

Benchmark run:

```text
target/release/ecaz --database postgres --host /tmp --port 28818 bench suite run --config reviews/task-142/017-release-n2048-extension/artifacts/task142-release-n2048-extension-suite.json --artifact-dir reviews/task-142/017-release-n2048-extension/artifacts --manifest-output reviews/task-142/017-release-n2048-extension/artifacts/suite-manifest.json --results-output reviews/task-142/017-release-n2048-extension/artifacts/suite-results.jsonl
```

## Primary Artifacts

| Artifact | Purpose |
| --- | --- |
| `task142-release-n2048-extension-suite.json` | Two-cell release suite config: 50k n2048/b0 and 100k n2048/b0. |
| `suite-manifest.json`, `suite-results.jsonl` | Top-level suite manifest/results for the two `spire-local-multinode` cells. |
| `suite-manifest-dry-run.json` | Dry-run expansion evidence. |
| `epoch-invalidation-test.log` | Focused Rust unit test log for the epoch-change routing cache regression. |
| `<cell>/local-multinode.log` | Four-node fixture setup and release-profile evidence. |
| `<cell>/bench-suite/suite-manifest.json` | Nested suite manifest with coordinator plus per-node backend build profiles. |
| `<cell>/bench-suite/results.jsonl` | Nested storage, query latency/recall, production-read profile, timeline, and per-node phase rows. |
| `<cell>/bench-suite/production-read-k10-default-default.log` | Human-readable production-read benchmark output for the 200-query sweep. |
| `<cell>/bench-suite/storage.log` | Coordinator storage output. |
| `<cell>/{coordinator-load,remote-load-node-2,remote-load-node-3,remote-load-node-4}.log` | Load and index-build timing evidence. |

Generated shard TSVs under `<cell>/distributed-correctness/node-*` are
regenerable corpus data and are intentionally not committed.

## Backend Provenance

Every accepted cell records release installation:

```text
install_profile=release
node_build_profile node_id=1 name=coord port=39800 profile=release
node_build_profile node_id=2 name=remote1 port=39801 profile=release
node_build_profile node_id=3 name=remote2 port=39802 profile=release
node_build_profile node_id=4 name=remote3 port=39803 profile=release
SPIRE local multinode fixture passed
HARNESS PASSED
```

Every cited `spire-pipeline` row in nested `results.jsonl` includes:

```text
backend_build_profile=release
backend_node_profiles=coordinator:39800:release,local-port-39801:39801:release,local-port-39802:39802:release,local-port-39803:39803:release
```

## Epoch-Invalidation Regression

`epoch-invalidation-test.log`:

```text
test am::ec_spire::scan::tests::collect_cached_resolved_scan_plan_selection_reloads_on_epoch_change ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2261 filtered out
```

The test warms the routing hierarchy cache at `(index_relid, active_epoch=7)`,
then reads the same index relid at `active_epoch=8` with a different hierarchy
and asserts both a reload (`routing_hierarchy_load_count=1`) and the new epoch's
selected leaf PID.

## Query Latency and Recall

Rows below are end-to-end production-read query rows from nested
`bench-suite/results.jsonl`.

| Cell | nprobe | query p50 | query p95 | recall@10 | distinct recall@10 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 50k n2048/b0 | 8 | 65.723 ms | 67.795 ms | 0.6625 | 0.6625 |
| 50k n2048/b0 | 16 | 65.915 ms | 67.629 ms | 0.7665 | 0.7665 |
| 50k n2048/b0 | 32 | 67.854 ms | 71.188 ms | 0.8550 | 0.8550 |
| 50k n2048/b0 | 64 | 69.195 ms | 72.021 ms | 0.9135 | 0.9135 |
| 50k n2048/b0 | 96 | 67.143 ms | 68.716 ms | 0.9405 | 0.9405 |
| 100k n2048/b0 | 8 | 66.170 ms | 69.200 ms | 0.6000 | 0.6000 |
| 100k n2048/b0 | 16 | 66.303 ms | 68.235 ms | 0.7195 | 0.7195 |
| 100k n2048/b0 | 32 | 66.869 ms | 69.309 ms | 0.8185 | 0.8185 |
| 100k n2048/b0 | 64 | 67.923 ms | 69.740 ms | 0.8840 | 0.8840 |
| 100k n2048/b0 | 96 | 69.311 ms | 73.781 ms | 0.9175 | 0.9175 |

## Production-Read Profile Rows

Profile totals are attribution rows from nested `results.jsonl`; they are not
the end-to-end query p50.

| Cell | nprobe | total p50 | manifest load p50 | leaf count p50 | route select p50 | manifest hits | manifest misses | routing loads | pool hits | socket opens | endpoint identity queries |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 50k n2048/b0 | 8 | 35.790 ms | 0.022 ms | 1.152 ms | 4.401 ms | 200 | 0 | 0 | 574 | 0 | 0 |
| 50k n2048/b0 | 16 | 36.079 ms | 0.023 ms | 1.150 ms | 4.389 ms | 200 | 0 | 0 | 600 | 0 | 0 |
| 50k n2048/b0 | 32 | 37.582 ms | 0.023 ms | 1.195 ms | 4.435 ms | 200 | 0 | 0 | 600 | 0 | 0 |
| 50k n2048/b0 | 64 | 38.769 ms | 0.023 ms | 1.234 ms | 4.457 ms | 200 | 0 | 0 | 600 | 0 | 0 |
| 50k n2048/b0 | 96 | 37.612 ms | 0.023 ms | 1.148 ms | 4.397 ms | 200 | 0 | 0 | 600 | 0 | 0 |
| 100k n2048/b0 | 8 | 36.275 ms | 0.023 ms | 1.178 ms | 4.393 ms | 200 | 0 | 0 | 579 | 0 | 0 |
| 100k n2048/b0 | 16 | 36.668 ms | 0.023 ms | 1.174 ms | 4.385 ms | 200 | 0 | 0 | 600 | 0 | 0 |
| 100k n2048/b0 | 32 | 37.445 ms | 0.023 ms | 1.181 ms | 4.407 ms | 200 | 0 | 0 | 600 | 0 | 0 |
| 100k n2048/b0 | 64 | 38.523 ms | 0.023 ms | 1.185 ms | 4.390 ms | 200 | 0 | 0 | 600 | 0 | 0 |
| 100k n2048/b0 | 96 | 40.022 ms | 0.023 ms | 1.191 ms | 4.413 ms | 200 | 0 | 0 | 600 | 0 | 0 |

Compared with packet 016 n1024 post-cache rows, the true route-select descent
roughly doubles from ~2.13 ms to ~4.40 ms, while redundant load stays eliminated:
`manifest_cache_hit_sum=200`, `manifest_cache_miss_sum=0`,
`routing_hierarchy_load_sum=0`, `socket_open_sum=0`, and
`endpoint_identity_query_sum=0` for every cited n2048 row.

## Storage

| Cell | coordinator SPIRE index | reloptions | leaf assignments | mean replicas | object count |
| --- | ---: | --- | ---: | ---: | ---: |
| 50k n2048/b0 | 63.6 MiB | `{nlists=2048,boundary_replica_count=0,source_identity=include,storage_format=rabitq}` | 50000 | 1.0000 | 2049 |
| 100k n2048/b0 | 108.1 MiB | `{nlists=2048,boundary_replica_count=0,source_identity=include,storage_format=rabitq}` | 100000 | 1.0000 | 2049 |

## Load / Build Timing

| Cell | coordinator build | coordinator total | remote2 build / total | remote3 build / total | remote4 build / total |
| --- | ---: | ---: | ---: | ---: | ---: |
| 50k n2048/b0 | 323.22s | 367.37s | 113.20s / 126.54s | 114.54s / 127.95s | 114.22s / 127.66s |
| 100k n2048/b0 | 634.61s | 726.79s | 222.48s / 253.21s | 212.73s / 240.57s | 207.52s / 233.15s |

## Notes

- Packet 016 already provides the 10k/50k/100k post-cache release A/B anchor at
  nlists 128 and 1024. This packet supplies the missing `nlists=2048` 50k/100k
  extension requested by the packet 016 reviewer.
- There is no Task 141 pre-cache `nlists=2048` before-row to compare directly.
  The relevant 2048 claim here is the cache invariant itself: steady-state rows
  still show zero routing hierarchy loads and zero manifest misses, with only the
  expected true routing descent increasing from n1024 to n2048.
