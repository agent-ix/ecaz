# Task 145 Packet 004 Artifact Manifest

Task bucket: `reviews/task-145/004-remote-rerank-width-ab`
Branch: `task-145-spire-rerank-economy-low-probe`
Head SHA: `c7ed28c9ff16f02aee6d5ecc41a05fdd8f2c7909`
Recorded: `2026-07-06T12:57:15Z`

## Scope

Remote-path release A/B for `ec_spire.rerank_width=0` versus
`ec_spire.rerank_width=50` after the packet 003 code checkpoint
(`4d7e927f0`) taught the remote coordinator path to propagate and honor the
effective rerank width.

This packet responds to reviewer feedback in packet 003: run the real remote
path with `remote_fanout_sum > 0`, release build evidence, latency, and recall.

## Commands

Dry run:

```sh
target/release/ecaz bench suite run \
  --config reviews/task-145/004-remote-rerank-width-ab/artifacts/task145-remote-rerank-width-ab-suite.json \
  --dry-run \
  --manifest-output reviews/task-145/004-remote-rerank-width-ab/artifacts/suite-manifest-dry-run-r3.json \
  --log-file reviews/task-145/004-remote-rerank-width-ab/artifacts/suite-dry-run-r3.log
```

Audit:

```sh
target/release/ecaz bench suite audit \
  --config reviews/task-145/004-remote-rerank-width-ab/artifacts/task145-remote-rerank-width-ab-suite.json \
  --log-file reviews/task-145/004-remote-rerank-width-ab/artifacts/suite-audit-r3.log
```

Evidence run:

```sh
target/release/ecaz bench suite run \
  --config reviews/task-145/004-remote-rerank-width-ab/artifacts/task145-remote-rerank-width-ab-suite.json \
  --manifest-output reviews/task-145/004-remote-rerank-width-ab/artifacts/suite-manifest-r3.json \
  --results-output reviews/task-145/004-remote-rerank-width-ab/artifacts/suite-results-r3.jsonl \
  --log-file reviews/task-145/004-remote-rerank-width-ab/artifacts/suite-run-r3.log
```

The top-level suite step launches nested `spire-local-multinode` suites, so the
per-scale result rows are in the nested `bench-suite/results.jsonl` files listed
below. The top-level `suite-results-r3.jsonl` is empty.

## Artifacts

- `task145-remote-rerank-width-ab-suite.json` - owning `SuiteConfig`.
- `suite-manifest-dry-run-r3.json`, `suite-dry-run-r3.log` - dry-run output.
- `suite-audit-r3.log` - audit output.
- `suite-manifest-r3.json`, `suite-run-r3.log`, `suite-results-r3.jsonl` -
  top-level evidence run outputs.
- `remote-10k-n128-r3/local-multinode.log` - 10k release install and node build profiles.
- `remote-10k-n128-r3/bench-suite/local-real-production-read-suite.json` - nested suite config.
- `remote-10k-n128-r3/bench-suite/suite-manifest.json` - nested suite manifest.
- `remote-10k-n128-r3/bench-suite/results.jsonl` - nested result rows.
- `remote-10k-n128-r3/bench-suite/storage.log` - nested storage step output.
- `remote-10k-n128-r3/bench-suite/suite-run.log` - nested suite run log.
- `remote-10k-n128-r3/bench-suite/production-read-k10-rerank-full-default.log` - width 0 production-read log.
- `remote-10k-n128-r3/bench-suite/production-read-k10-rerank-50-default.log` - width 50 production-read log.
- `remote-10k-n128-r3/bench-suite/production-read-k10-rerank-full-default-identity.jsonl` - width 0 identity rows.
- `remote-10k-n128-r3/bench-suite/production-read-k10-rerank-50-default-identity.jsonl` - width 50 identity rows.
- Same nested files under `remote-50k-n1024-r3/` and `remote-100k-n1024-r3/`.

Corpus TSVs, generated distributed-correctness TSVs, PostgreSQL server logs, and
other local-multinode exhaust were not staged as review evidence.

## Fixture

All cells use isolated one-index-per-table surfaces through `spire-local-multinode`.

| scale | prefix | nlists | run_id | storage | variants |
| --- | --- | ---: | --- | --- | --- |
| 10k | `ec_real_10k` | 128 | `t145r4-10n128` | `rabitq` | `rerank_width={0,50}` |
| 50k | `ec_real_50k` | 1024 | `t145r4-50n1024` | `rabitq` | `rerank_width={0,50}` |
| 100k | `ec_real_100k` | 1024 | `t145r4-100n1024` | `rabitq` | `rerank_width={0,50}` |

Held GUCs / options:

- `ec_spire.leaf_score_only_routing=on`
- `ec_spire.route_overfetch_multiplier=1.0`
- `ec_spire.probe_distance_ratio=0`
- `ec_spire.max_remote_payload_bytes_per_row=16384`
- `projection=id,source`
- `--include-remote --require-remote-placements`
- `--include-production-read-profile --production-read-only`
- `--include-recall --truth-corpus-file data/staged-current/<prefix>_corpus.tsv`

## Release Build Evidence

Each successful r3 `local-multinode.log` records:

```text
install_profile=release
node_build_profile node_id=1 name=coord port=39700 profile=release
node_build_profile node_id=2 name=remote1 port=39701 profile=release
node_build_profile node_id=3 name=remote2 port=39702 profile=release
node_build_profile node_id=4 name=remote3 port=39703 profile=release
```

Each nested `results.jsonl` also records
`backend_build_profile=release` and
`backend_node_profiles=coordinator:39700:release,local-port-39701:39701:release,local-port-39702:39702:release,local-port-39703:39703:release`.

## Key Results

Pipeline recall is equal between width 0 and width 50 at every measured
`nprobe`. The identity JSONL files are byte-identical between variants at all
three scales (`cmp` exit status 0), so the packet 002 pipeline/truth-cache
contradiction did not reproduce here.

The remote path is engaged: `remote_heap_candidate_sum` is nonzero and
`local_heap_candidate_sum=0` for every profile row. However, width 50 did not
reduce `remote_heap_candidate_sum` versus width 0 in this production-read run.
The end-to-end win remains unproven.

### nprobe=96 Summary

| scale | width | distinct_recall@k | pipeline p50/p95 | profile total p50/p95 | remote_heap_candidate_sum | local_heap_candidate_sum | remote_pid_sum |
| --- | ---: | ---: | --- | --- | ---: | ---: | ---: |
| 10k n128 | 0 | 0.9855 | 64.556 / 67.950 ms | 34.053 / 36.269 ms | 6000 | 0 | 19200 |
| 10k n128 | 50 | 0.9855 | 63.256 / 67.006 ms | 33.227 / 36.068 ms | 6000 | 0 | 19200 |
| 50k n1024 | 0 | 0.9560 | 68.213 / 76.032 ms | 38.478 / 43.431 ms | 6000 | 0 | 19200 |
| 50k n1024 | 50 | 0.9560 | 70.605 / 72.780 ms | 40.212 / 42.411 ms | 6000 | 0 | 19200 |
| 100k n1024 | 0 | 0.9480 | 70.048 / 72.402 ms | 40.732 / 42.511 ms | 6000 | 0 | 19200 |
| 100k n1024 | 50 | 0.9480 | 69.947 / 72.280 ms | 40.470 / 42.463 ms | 6000 | 0 | 19200 |

### All nprobe Rows

10k n128:

| width | nprobe | distinct_recall@k | pipeline p50/p95 | profile total p50/p95 | remote_heap_candidate_sum | remote_pid_sum |
| ---: | ---: | ---: | --- | --- | ---: | ---: |
| 0 | 8 | 0.9790 | 59.018 / 61.236 ms | 28.973 / 30.182 ms | 5690 | 1600 |
| 50 | 8 | 0.9790 | 59.265 / 61.226 ms | 29.293 / 30.592 ms | 5690 | 1600 |
| 0 | 16 | 0.9825 | 59.437 / 61.176 ms | 29.489 / 30.529 ms | 6000 | 3200 |
| 50 | 16 | 0.9825 | 59.606 / 60.858 ms | 29.612 / 30.584 ms | 6000 | 3200 |
| 0 | 32 | 0.9855 | 60.212 / 62.652 ms | 30.475 / 32.152 ms | 6000 | 6400 |
| 50 | 32 | 0.9855 | 60.163 / 61.785 ms | 30.521 / 31.794 ms | 6000 | 6400 |
| 0 | 64 | 0.9855 | 63.552 / 65.895 ms | 33.288 / 35.313 ms | 6000 | 12800 |
| 50 | 64 | 0.9855 | 61.578 / 62.808 ms | 31.712 / 33.332 ms | 6000 | 12800 |
| 0 | 96 | 0.9855 | 64.556 / 67.950 ms | 34.053 / 36.269 ms | 6000 | 19200 |
| 50 | 96 | 0.9855 | 63.256 / 67.006 ms | 33.227 / 36.068 ms | 6000 | 19200 |

50k n1024:

| width | nprobe | distinct_recall@k | pipeline p50/p95 | profile total p50/p95 | remote_heap_candidate_sum | remote_pid_sum |
| ---: | ---: | ---: | --- | --- | ---: | ---: |
| 0 | 8 | 0.7555 | 65.698 / 68.541 ms | 35.366 / 37.879 ms | 5880 | 1600 |
| 50 | 8 | 0.7555 | 65.651 / 67.983 ms | 35.414 / 37.182 ms | 5880 | 1600 |
| 0 | 16 | 0.8455 | 65.717 / 67.325 ms | 35.523 / 36.931 ms | 6000 | 3200 |
| 50 | 16 | 0.8455 | 65.697 / 67.526 ms | 35.510 / 36.898 ms | 6000 | 3200 |
| 0 | 32 | 0.9070 | 66.132 / 67.992 ms | 36.040 / 37.673 ms | 6000 | 6400 |
| 50 | 32 | 0.9070 | 66.351 / 67.613 ms | 36.246 / 37.591 ms | 6000 | 6400 |
| 0 | 64 | 0.9440 | 69.404 / 72.582 ms | 38.947 / 40.964 ms | 6000 | 12800 |
| 50 | 64 | 0.9440 | 67.327 / 70.246 ms | 37.381 / 39.625 ms | 6000 | 12800 |
| 0 | 96 | 0.9560 | 68.213 / 76.032 ms | 38.478 / 43.431 ms | 6000 | 19200 |
| 50 | 96 | 0.9560 | 70.605 / 72.780 ms | 40.212 / 42.411 ms | 6000 | 19200 |

100k n1024:

| width | nprobe | distinct_recall@k | pipeline p50/p95 | profile total p50/p95 | remote_heap_candidate_sum | remote_pid_sum |
| ---: | ---: | ---: | --- | --- | ---: | ---: |
| 0 | 8 | 0.7070 | 66.751 / 69.661 ms | 36.261 / 38.580 ms | 5650 | 1600 |
| 50 | 8 | 0.7070 | 67.687 / 70.051 ms | 37.079 / 38.969 ms | 5650 | 1600 |
| 0 | 16 | 0.8185 | 65.957 / 69.556 ms | 35.980 / 38.234 ms | 6000 | 3200 |
| 50 | 16 | 0.8185 | 66.445 / 70.317 ms | 36.371 / 39.150 ms | 6000 | 3200 |
| 0 | 32 | 0.8810 | 66.637 / 68.317 ms | 36.902 / 38.430 ms | 6000 | 6400 |
| 50 | 32 | 0.8810 | 66.827 / 68.569 ms | 37.080 / 38.503 ms | 6000 | 6400 |
| 0 | 64 | 0.9285 | 68.273 / 70.035 ms | 38.716 / 40.510 ms | 6000 | 12800 |
| 50 | 64 | 0.9285 | 68.243 / 69.756 ms | 38.694 / 40.490 ms | 6000 | 12800 |
| 0 | 96 | 0.9480 | 70.048 / 72.402 ms | 40.732 / 42.511 ms | 6000 | 19200 |
| 50 | 96 | 0.9480 | 69.947 / 72.280 ms | 40.470 / 42.463 ms | 6000 | 19200 |

## Interpretation

This is release, remote-path evidence, not local-only evidence. It satisfies the
reviewer's requirement to stop claiming from packet 001's local path.

The result is negative for Task 145 AC1 promotion: `rerank_width=50` preserves
recall, but it does not reduce `remote_heap_candidate_sum` in this run, and
latency changes are small/noisy rather than the expected full-frontier to top-W
collapse. The next code slice should inspect whether the production-read remote
receive path is counting candidates before width truncation, whether the width
GUC is applied too late for the measured counter, or whether the remote SQL path
still bypasses the packet 003 truncation.

## Setup Notes

Earlier attempts were infrastructure-only retries and are not cited as evidence:

- r1 was interrupted before useful results while running inside the sandbox.
- r2 failed because the long run id made PostgreSQL Unix socket paths exceed the
  platform limit.
- r3 shortened run ids and completed successfully with exit status 0.
