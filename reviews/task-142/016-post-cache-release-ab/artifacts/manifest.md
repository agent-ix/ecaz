# Task 142 Packet 016 Artifact Manifest

- head SHA: `0ffd7c2da5fe55bdc688fa6d5f3d15ea7a3d9075`
- task bucket: `reviews/task-142/`
- packet path: `reviews/task-142/016-post-cache-release-ab/`
- timestamp: `2026-07-05`
- slice: post-cache release A/B anchor evidence against Task 141 release anchors
- lane / fixture / storage / rerank: local PG18 SPIRE multi-instance, one coordinator plus three local worker instances, staged real corpora, `source_identity=include`, `storage_format=rabitq`, `boundary_replica_count=0`, `rerank_width=0`
- isolated one-index-per-table vs shared-table surface: isolated local multinode fixture with one coordinator index and one remote index per remote node; not a shared-table run
- evidence runner: `ecaz bench suite`

## Commands

Release CLI build used for the suite:

```text
cargo build --release -p ecaz-cli
```

Dry run:

```text
target/release/ecaz --database postgres --host /tmp --port 28818 bench suite run --config reviews/task-142/016-post-cache-release-ab/artifacts/task142-post-cache-release-anchor-suite.json --dry-run --artifact-dir reviews/task-142/016-post-cache-release-ab/artifacts --manifest-output reviews/task-142/016-post-cache-release-ab/artifacts/suite-manifest-dry-run.json --results-output reviews/task-142/016-post-cache-release-ab/artifacts/suite-results-dry-run.jsonl
```

Initial four-cell run:

```text
target/release/ecaz --database postgres --host /tmp --port 28818 bench suite run --config reviews/task-142/016-post-cache-release-ab/artifacts/task142-post-cache-release-anchor-suite.json --artifact-dir reviews/task-142/016-post-cache-release-ab/artifacts --manifest-output reviews/task-142/016-post-cache-release-ab/artifacts/suite-manifest.json --results-output reviews/task-142/016-post-cache-release-ab/artifacts/suite-results.jsonl
```

The first run completed 10k n128, 50k n128, and 50k n1024, then failed the
100k n1024 coordinator load with `No space left on device`. After deleting
generated local-multinode `target/` run directories, the failed cell was
retried with:

```text
target/release/ecaz --database postgres --host /tmp --port 28818 bench suite run --config reviews/task-142/016-post-cache-release-ab/artifacts/task142-post-cache-release-anchor-suite.json --only release-100k-n1024-b0 --artifact-dir reviews/task-142/016-post-cache-release-ab/artifacts --manifest-output reviews/task-142/016-post-cache-release-ab/artifacts/suite-manifest-100k-retry.json --results-output reviews/task-142/016-post-cache-release-ab/artifacts/suite-results-100k-retry.jsonl
```

## Primary Artifacts

| Artifact | Purpose |
| --- | --- |
| `task142-post-cache-release-anchor-suite.json` | Four-cell release suite config: 10k n128/b0, 50k n128/b0, 50k n1024/b0, 100k n1024/b0. |
| `suite-manifest.json`, `suite-run.log` | Top-level initial run manifest/log. Records the disk-space failure for the 100k cell after the first three cells succeeded. |
| `suite-manifest-100k-retry.json`, `suite-run-100k-retry.log` | One-cell retry manifest/log for the accepted 100k n1024/b0 result. |
| `<cell>/local-multinode.log` | Four-node fixture setup and release-profile evidence for each cited cell. |
| `<cell>/bench-suite/suite-manifest.json` | Nested suite manifest with coordinator plus per-node backend build profiles. |
| `<cell>/bench-suite/results.jsonl` | Nested storage, query latency/recall, production-read profile, timeline, and per-node phase rows. |
| `<cell>/bench-suite/production-read-k10-default-default.log` | Human-readable production-read benchmark output for each cited cell. |
| `<cell>/bench-suite/storage.log` | Coordinator storage output for each cited cell. |
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

## Query Latency and Recall

Rows below are end-to-end production-read query rows from nested
`bench-suite/results.jsonl`.

| Cell | nprobe | query p50 | query p95 | recall@10 | distinct recall@10 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k n128/b0 | 8 | 56.482 ms | 58.334 ms | 0.9790 | 0.9790 |
| 10k n128/b0 | 16 | 57.049 ms | 58.663 ms | 0.9825 | 0.9825 |
| 10k n128/b0 | 32 | 57.604 ms | 59.687 ms | 0.9855 | 0.9855 |
| 10k n128/b0 | 64 | 58.986 ms | 61.026 ms | 0.9855 | 0.9855 |
| 10k n128/b0 | 96 | 61.043 ms | 64.821 ms | 0.9855 | 0.9855 |
| 50k n128/b0 | 8 | 57.929 ms | 60.083 ms | 0.8920 | 0.8920 |
| 50k n128/b0 | 16 | 59.357 ms | 61.693 ms | 0.9375 | 0.9375 |
| 50k n128/b0 | 32 | 62.769 ms | 65.747 ms | 0.9725 | 0.9725 |
| 50k n128/b0 | 64 | 70.779 ms | 75.719 ms | 0.9865 | 0.9865 |
| 50k n128/b0 | 96 | 77.538 ms | 86.675 ms | 0.9900 | 0.9900 |
| 50k n1024/b0 | 8 | 60.469 ms | 62.682 ms | 0.7240 | 0.7240 |
| 50k n1024/b0 | 16 | 60.892 ms | 62.854 ms | 0.8210 | 0.8210 |
| 50k n1024/b0 | 32 | 61.402 ms | 62.854 ms | 0.8895 | 0.8895 |
| 50k n1024/b0 | 64 | 62.490 ms | 64.831 ms | 0.9375 | 0.9375 |
| 50k n1024/b0 | 96 | 65.212 ms | 68.510 ms | 0.9575 | 0.9575 |
| 100k n1024/b0 | 8 | 60.635 ms | 63.667 ms | 0.6715 | 0.6715 |
| 100k n1024/b0 | 16 | 61.287 ms | 64.219 ms | 0.7860 | 0.7860 |
| 100k n1024/b0 | 32 | 62.392 ms | 65.062 ms | 0.8615 | 0.8615 |
| 100k n1024/b0 | 64 | 66.901 ms | 69.327 ms | 0.9105 | 0.9105 |
| 100k n1024/b0 | 96 | 67.684 ms | 72.138 ms | 0.9355 | 0.9355 |

Recall is unchanged from the Task 141 release anchors for matched 50k/100k
cells.

## Task 141 Release Comparison

Matched Task 141 anchors are from
`reviews/task-141/001-release-anchor-rebaseline/artifacts/manifest.md`.

| Cell | nprobe | Task 141 p50 | Task 142 p50 | p50 delta |
| --- | ---: | ---: | ---: | ---: |
| 50k n128/b0 | 8 | 64.966 ms | 57.929 ms | -10.8% |
| 50k n128/b0 | 16 | 66.694 ms | 59.357 ms | -11.0% |
| 50k n128/b0 | 32 | 69.993 ms | 62.769 ms | -10.3% |
| 50k n128/b0 | 64 | 77.131 ms | 70.779 ms | -8.2% |
| 50k n128/b0 | 96 | 85.229 ms | 77.538 ms | -9.0% |
| 50k n1024/b0 | 8 | 109.010 ms | 60.469 ms | -44.5% |
| 50k n1024/b0 | 16 | 107.727 ms | 60.892 ms | -43.5% |
| 50k n1024/b0 | 32 | 107.254 ms | 61.402 ms | -42.8% |
| 50k n1024/b0 | 64 | 108.127 ms | 62.490 ms | -42.2% |
| 50k n1024/b0 | 96 | 110.109 ms | 65.212 ms | -40.8% |
| 100k n1024/b0 | 8 | 104.843 ms | 60.635 ms | -42.2% |
| 100k n1024/b0 | 16 | 106.654 ms | 61.287 ms | -42.5% |
| 100k n1024/b0 | 32 | 108.299 ms | 62.392 ms | -42.4% |
| 100k n1024/b0 | 64 | 113.852 ms | 66.901 ms | -41.2% |
| 100k n1024/b0 | 96 | 111.314 ms | 67.684 ms | -39.2% |

## Production-Read Profile Rows

Profile totals are attribution rows from nested `results.jsonl`; they are not
the end-to-end query p50.

| Cell | nprobe | total p50 | manifest load p50 | leaf count p50 | route select p50 | manifest cache hits | pool hits | routing loads |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k n128/b0 | 8 | 27.700 ms | 0.002 ms | 0.082 ms | 0.263 ms | 200 | 569 | 0 |
| 10k n128/b0 | 16 | 28.186 ms | 0.002 ms | 0.081 ms | 0.264 ms | 200 | 600 | 0 |
| 10k n128/b0 | 32 | 28.953 ms | 0.002 ms | 0.081 ms | 0.267 ms | 200 | 600 | 0 |
| 10k n128/b0 | 64 | 30.170 ms | 0.002 ms | 0.081 ms | 0.268 ms | 200 | 600 | 0 |
| 10k n128/b0 | 96 | 32.059 ms | 0.002 ms | 0.083 ms | 0.270 ms | 200 | 600 | 0 |
| 50k n128/b0 | 8 | 29.537 ms | 0.002 ms | 0.077 ms | 0.263 ms | 200 | 593 | 0 |
| 50k n128/b0 | 16 | 31.182 ms | 0.002 ms | 0.076 ms | 0.263 ms | 200 | 600 | 0 |
| 50k n128/b0 | 32 | 34.617 ms | 0.002 ms | 0.076 ms | 0.264 ms | 200 | 600 | 0 |
| 50k n128/b0 | 64 | 42.427 ms | 0.002 ms | 0.077 ms | 0.270 ms | 200 | 600 | 0 |
| 50k n128/b0 | 96 | 49.595 ms | 0.002 ms | 0.079 ms | 0.270 ms | 200 | 600 | 0 |
| 50k n1024/b0 | 8 | 31.356 ms | 0.011 ms | 0.631 ms | 2.132 ms | 200 | 581 | 0 |
| 50k n1024/b0 | 16 | 31.850 ms | 0.011 ms | 0.621 ms | 2.122 ms | 200 | 600 | 0 |
| 50k n1024/b0 | 32 | 32.376 ms | 0.011 ms | 0.619 ms | 2.122 ms | 200 | 600 | 0 |
| 50k n1024/b0 | 64 | 33.625 ms | 0.011 ms | 0.615 ms | 2.124 ms | 200 | 600 | 0 |
| 50k n1024/b0 | 96 | 36.231 ms | 0.011 ms | 0.645 ms | 2.138 ms | 200 | 600 | 0 |
| 100k n1024/b0 | 8 | 31.578 ms | 0.011 ms | 0.619 ms | 2.131 ms | 200 | 552 | 0 |
| 100k n1024/b0 | 16 | 32.543 ms | 0.011 ms | 0.609 ms | 2.132 ms | 200 | 595 | 0 |
| 100k n1024/b0 | 32 | 33.878 ms | 0.011 ms | 0.603 ms | 2.147 ms | 200 | 600 | 0 |
| 100k n1024/b0 | 64 | 37.753 ms | 0.011 ms | 0.653 ms | 2.164 ms | 200 | 600 | 0 |
| 100k n1024/b0 | 96 | 38.991 ms | 0.011 ms | 0.645 ms | 2.186 ms | 200 | 600 | 0 |

Task 141 release profile-total p50 comparison:

- 50k n128/b0 improved from 33/34/38/45/52 ms to
  29.537/31.182/34.617/42.427/49.595 ms at nprobe 8/16/32/64/96.
- 50k n1024/b0 improved from 49/50/49/51/52 ms to
  31.356/31.850/32.376/33.625/36.231 ms.
- 100k n1024/b0 improved from 48/49/51/55/54 ms to
  31.578/32.543/33.878/37.753/38.991 ms.

## Storage

| Cell | coordinator SPIRE index | reloptions |
| --- | ---: | --- |
| 10k n128/b0 | 10.0 MiB | `{nlists=128,boundary_replica_count=0,source_identity=include,storage_format=rabitq}` |
| 50k n128/b0 | 44.8 MiB | `{nlists=128,boundary_replica_count=0,source_identity=include,storage_format=rabitq}` |
| 50k n1024/b0 | 54.1 MiB | `{nlists=1024,boundary_replica_count=0,source_identity=include,storage_format=rabitq}` |
| 100k n1024/b0 | 97.5 MiB | `{nlists=1024,boundary_replica_count=0,source_identity=include,storage_format=rabitq}` |

## Notes

- The first 100k attempt failed from host disk exhaustion before measurement;
  the accepted 100k result is from `suite-manifest-100k-retry.json`.
- Top-level `suite-results*.jsonl` files are empty because
  `spire-local-multinode` writes measured rows through each nested
  `bench-suite/results.jsonl`.
- The branch also contains earlier packet 015 microsecond timer changes, so
  profile attribution rows in this packet use `*_elapsed_us` internally and are
  rendered as sub-millisecond values where applicable.
