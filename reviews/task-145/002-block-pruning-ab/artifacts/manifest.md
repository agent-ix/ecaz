# Task 145 Packet 002 Artifact Manifest

- Head SHA: `bdbe3e6f3a8529a2708b0029cdcd1ac0c33349ff`
- Branch: `task-145-spire-rerank-economy-low-probe`
- Task bucket: `reviews/task-145/002-block-pruning-ab`
- Timestamp: `2026-07-06T11:30:36Z`
- Runner: `target/release/ecaz bench suite run --config reviews/task-145/002-block-pruning-ab/artifacts/task145-block-pruning-ab-suite.json --database tqvector_bench_task145 --host /home/peter/dev/ecaz/target/task145-pg18-socket --port 28818 --artifact-dir reviews/task-145/002-block-pruning-ab/artifacts --manifest-output reviews/task-145/002-block-pruning-ab/artifacts/suite-manifest.json --results-output reviews/task-145/002-block-pruning-ab/artifacts/suite-results.jsonl --log-file reviews/task-145/002-block-pruning-ab/artifacts/suite-run.log`
- Backend node: coordinator, database `tqvector_bench_task145`, host `/home/peter/dev/ecaz/target/task145-pg18-socket`, port `28818`, build profile `release`
- Installed extension: `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`, sha256 `a821e3ee67501cc7489dcc9380e2bfab867b33388f600ef1f8109d19751a5bf8`
- Isolation: one index per prefix/table; paired control/treatment prefixes per scale.
- Baseline controls: `rerank_width=50`, `ec_spire.leaf_score_only_routing=on`, `ec_spire.route_overfetch_multiplier=1.0`, `ec_spire.probe_distance_ratio=0`, block pruning disabled.
- Treatment build options: `ec_spire.leaf_block_rows=16`, `ec_spire.leaf_block_summary_representatives=2`.
- Treatment scan options: `ec_spire.leaf_block_pruning_max_global_blocks=128`, `ec_spire.leaf_block_pruning_max_blocks_per_leaf=0`, `ec_spire.leaf_block_pruning_global_probe_blocks=0`, `ec_spire.leaf_block_pruning_sample_rows_per_block=0`, `ec_spire.leaf_block_pruning_summary_radius_weight=1.0`, `ec_spire.leaf_block_pruning_route_prior_weight=0.0`.

## Artifacts

- `task145-block-pruning-ab-suite.json`: checked suite configuration for the 10k / 50k / 100k A/B.
- `suite-manifest.json`: suite execution manifest with release backend node profile.
- `suite-results.jsonl`: structured suite output used for the tables below.
- `suite-run.log`: top-level suite log.
- `load-*.log`, `storage-*.log`, `truth-cache-*.log`, `pipeline-*.log`: packet-local command logs for each suite step.
- `suite-manifest-dry-run.json`: dry-run manifest from preflight.
- `truth-cache-*.json`: generated truth caches are intentionally not committed; they are gitignored regenerable artifacts.
- `leaf-block-rank-*.jsonl`: generated per-query rank dumps are intentionally not committed; they are bulky diagnostic exhaust. Candidate sums below prove pruning engaged.

## Key Results

Pipeline latency and recall:

| scale | variant | nprobe | p50 | p95 | distinct recall@k |
| --- | --- | ---: | ---: | ---: | ---: |
| 10k n128 | control | 96 | 9.466 ms | 10.238 ms | 1.0000 |
| 10k n128 | block | 96 | 6.267 ms | 6.942 ms | 0.9920 |
| 50k n1024 | control | 96 | 13.619 ms | 14.805 ms | 0.9595 |
| 50k n1024 | block | 96 | 14.103 ms | 16.329 ms | 0.9085 |
| 100k n1024 | control | 96 | 17.482 ms | 19.179 ms | 0.9570 |
| 100k n1024 | block | 96 | 12.826 ms | 15.438 ms | 0.7755 |

Recall truth-cache checks at nprobe96:

| scale | variant | distinct recall@k | mean q-time | backend |
| --- | --- | ---: | ---: | --- |
| 10k n128 | control | 1.0000 | 266.83 ms | release |
| 10k n128 | block | 1.0000 | 266.31 ms | release |
| 50k n1024 | control | 0.9590 | 187.45 ms | release |
| 50k n1024 | block | 0.9590 | 185.07 ms | release |
| 100k n1024 | control | 0.9300 | 401.21 ms | release |
| 100k n1024 | block | 0.9300 | 385.21 ms | release |

Storage:

| scale | variant | index size | per row |
| --- | --- | ---: | ---: |
| 10k n128 | control | 10.1 MiB | 1058.4 B |
| 10k n128 | block | 11.4 MiB | 1196.0 B |
| 50k n1024 | control | 54.4 MiB | 1139.8 B |
| 50k n1024 | block | 61.0 MiB | 1278.8 B |
| 100k n1024 | control | 97.8 MiB | 1025.1 B |
| 100k n1024 | block | 110.9 MiB | 1163.0 B |

Candidate sums at nprobe96:

| scale | control candidates | block candidates |
| --- | ---: | ---: |
| 10k n128 | 1502699 | 396050 |
| 50k n1024 | 986258 | 395052 |
| 100k n1024 | 1874885 | 402835 |

## Decision

Do not promote `leaf_block_pruning_max_global_blocks=128` with `rerank_width=50`.
The treatment is mechanically active and can reduce candidate work, but it causes
unacceptable recall loss at high probes, including 50k nprobe96 0.9595 -> 0.9085
and 100k nprobe96 0.9570 -> 0.7755. It also regresses 50k p95. Keep the packet
as negative A/B evidence and continue to treat packet 001's `rerank_width=50`
as the only Task 145 promotion candidate so far.
