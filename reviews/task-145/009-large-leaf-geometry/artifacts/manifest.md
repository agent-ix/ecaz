# Task 145 Packet 009 Artifact Manifest

- Task: 145 - SPIRE scan/rerank economy at low probe counts
- Packet: `reviews/task-145/009-large-leaf-geometry`
- Head SHA: `0f39fdbe56cd4f4cdcb179b8c65852748b7a2c0c`
- Date: 2026-07-06

## Feedback Context

Task 145 packet 008 reviewer feedback landed before this packet was packaged.
Packet 008 is treated here as an inert/null bound-prune A/B: the raw data were
honest and the do-not-promote call was correct, but the bound-prune mechanism
did not engage on the remote path. AC2 bound-prune remains open until there is
a dedicated engagement counter, a runtime fix, and a real re-run.

This packet addresses the separate AC3 large-leaf geometry cell.

## Suite

- Runner: `target/release/ecaz bench suite`
- Harness step kind: `spire-local-multinode`
- PostgreSQL: PG18
- Install/build profile: release
- Scale: `100k`
- Boundary replicas: `0`
- Storage format: `rabitq`
- Query limit: 200
- `top_k`: 10
- `nprobe`: 8, 16, 32, 64, 96
- Cells:
  - control: `nlists=1024`
  - large leaf: `nlists=128`
- Common reloptions:
  - `recursive_fanout=8`
  - `top_graph_enabled=1`
  - `top_graph_degree=32`
  - `top_graph_build_list_size=100`
  - `top_graph_search_list_size=96`
  - `boundary_replica_count=0`
  - `training_sample_rows=100000`
  - `source_identity=include`
- Load GUCs:
  - `ec_spire.leaf_block_rows=16`
  - `ec_spire.leaf_block_summary_representatives=2`
- Production read variants in both cells:
  - `geometry-fixed`: `ec_spire.probe_distance_ratio=0`
  - `geometry-ratio4`: `ec_spire.probe_distance_ratio=4.0`
  - `geometry-ratio8`: `ec_spire.probe_distance_ratio=8.0`
- Common scan GUCs:
  - `ec_spire.leaf_score_only_routing=on`
  - `ec_spire.route_overfetch_multiplier=1.0`
  - `ec_spire.rerank_width=50`
  - `ec_spire.max_candidate_rows=100`
  - `ec_spire.leaf_block_pruning_max_blocks_per_leaf=0`
  - `ec_spire.leaf_block_pruning_max_global_blocks=128`
  - `ec_spire.leaf_block_pruning_global_probe_blocks=0`
  - `ec_spire.leaf_block_pruning_sample_rows_per_block=0`
  - `ec_spire.leaf_block_pruning_summary_radius_weight=1.0`
  - `ec_spire.leaf_block_pruning_route_prior_weight=0.0`
  - `ec_spire.max_remote_payload_bytes_per_row=16384`

## Commands

Audit:

```bash
target/release/ecaz bench suite audit --config reviews/task-145/009-large-leaf-geometry/artifacts/task145-large-leaf-geometry-suite.json --log-file reviews/task-145/009-large-leaf-geometry/artifacts/suite-audit.log
```

Dry run:

```bash
target/release/ecaz bench suite run --dry-run --config reviews/task-145/009-large-leaf-geometry/artifacts/task145-large-leaf-geometry-suite.json --artifact-dir reviews/task-145/009-large-leaf-geometry/artifacts --manifest-output reviews/task-145/009-large-leaf-geometry/artifacts/suite-manifest-dry-run.json --results-output reviews/task-145/009-large-leaf-geometry/artifacts/suite-results-dry-run.jsonl --log-file reviews/task-145/009-large-leaf-geometry/artifacts/suite-dry-run.log
```

Run:

```bash
target/release/ecaz bench suite run --config reviews/task-145/009-large-leaf-geometry/artifacts/task145-large-leaf-geometry-suite.json --artifact-dir reviews/task-145/009-large-leaf-geometry/artifacts --manifest-output reviews/task-145/009-large-leaf-geometry/artifacts/suite-manifest.json --results-output reviews/task-145/009-large-leaf-geometry/artifacts/suite-results.jsonl --log-file reviews/task-145/009-large-leaf-geometry/artifacts/suite-run.log
```

## Artifacts

- `task145-large-leaf-geometry-suite.json`: checked-in suite config.
- `suite-audit.log`: audit passed for 2 steps.
- `suite-dry-run.log`: dry-run expansion.
- `suite-manifest-dry-run.json`: dry-run manifest.
- `suite-run.log`: top-level suite run log.
- `suite-manifest.json`: top-level suite manifest.
- `suite-results.jsonl`: empty top-level sink for nested local-multinode steps.
- `remote-100k-n1024-control-r1/local-multinode.log`: release install/build and
  harness proof for the control.
- `remote-100k-n128-large-r1/local-multinode.log`: release install/build and
  harness proof for the large-leaf cell.
- `remote-*/bench-suite/local-real-production-read-suite.json`: nested emitted
  suite configs.
- `remote-*/bench-suite/suite-manifest.json`: nested suite manifests.
- `remote-*/bench-suite/suite-run.log`: nested suite run logs.
- `remote-*/bench-suite/results.jsonl`: nested result rows, 360 per cell.
- `remote-*/bench-suite/storage.log`: storage measurements.
- `remote-*/bench-suite/production-read-k10-geometry-*-default.log`: detailed
  production-read logs.
- `remote-*/bench-suite/production-read-k10-geometry-*-default-identity.jsonl`:
  1000-row identity traces per variant.
- `summary-n1024-recall-latency.txt`, `summary-n128-recall-latency.txt`:
  compact recall/latency summaries.
- `summary-n1024-frontier.txt`, `summary-n128-frontier.txt`: compact frontier
  summaries.
- `summary-leaf-counters-aggregate.txt`: summed remote leaf/block counters.

Generated corpus TSVs, distributed correctness TSVs, PostgreSQL server logs,
load logs, registration logs, and remote materialization logs are intentionally
not committed.

## Release Proof

Both cells record release install/build profiles and harness success:

```text
remote-100k-n1024-control-r1/local-multinode.log: install_profile=release; node_build_profile release for coord/remote1/remote2/remote3; HARNESS PASSED
remote-100k-n128-large-r1/local-multinode.log: install_profile=release; node_build_profile release for coord/remote1/remote2/remote3; HARNESS PASSED
```

Each nested result row also reports `backend_build_profile=release` and
`backend_node_profiles` with all nodes in release.

## Key Results

Best recall per geometry:

| cell | variant | nprobe | recall@10 | p50 | p95 |
| --- | --- | ---: | ---: | ---: | ---: |
| n1024 control | ratio8 | 96 | 0.9340 | 143.658 ms | 150.947 ms |
| n128 large-leaf | ratio4 | 16 | 0.8480 | 139.554 ms | 144.360 ms |
| n128 large-leaf | ratio8 | 16 | 0.8480 | 136.718 ms | 141.952 ms |
| n128 large-leaf | fixed | 96 | 0.7840 | 142.198 ms | 151.332 ms |

At nprobe96, the n128 large-leaf cell is faster only by noise-sized p50 and loses
15 recall points:

| cell | variant | recall@10 | p50 | p95 |
| --- | --- | ---: | ---: | ---: |
| n1024 control | fixed | 0.9340 | 145.738 ms | 152.307 ms |
| n128 large-leaf | fixed | 0.7840 | 142.198 ms | 151.332 ms |

Block-pruning counters are engaged in both cells. At nprobe96:

| cell | variant | blocks available | blocks skipped | leaf candidates | candidate score nanos |
| --- | --- | ---: | ---: | ---: | ---: |
| n1024 control | fixed | 126,184 | 49,387 | 1,196,856 | 622,545,820 |
| n128 large-leaf | fixed | 964,963 | 888,163 | 1,228,150 | 1,375,809,142 |

The n128 cell forces far more block-summary scoring and skips many blocks, but
the remote heap frontier still saturates at 30,000 candidates from nprobe16
upward and recall deteriorates as nprobe increases.

Storage:

| cell | SPIRE index size | indexes total | replication objects |
| --- | ---: | ---: | ---: |
| n1024 control | 111.0 MiB | 113.2 MiB | 1,034 |
| n128 large-leaf | 98.5 MiB | 100.7 MiB | 138 |

## Decision

Drop this `100k n128/b0` large-leaf shape for Task 145. It saves about 12.5 MiB
of SPIRE index storage and keeps p50 in the same transport-dominated range, but
it is not recall-competitive: best measured recall is 0.8480 and nprobe96 falls
to 0.7840, while the n1024 control reaches 0.9340. The large-leaf geometry does
exercise block pruning heavily, but the work shifts into leaf/block scoring
without producing a usable recall/latency point.
