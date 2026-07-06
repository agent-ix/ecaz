# Task 145 Packet 001: Rerank Width A/B

## Request

Please review the Task 145 Phase 0 release A/B for `ec_spire.rerank_width`.
This packet tests whether a bounded rerank width can remove the full-rerank
latency ramp without changing recall on the post-143/144 baseline.

## What Ran

- Runner: `ecaz bench suite`.
- Suite config:
  `reviews/task-145/001-rerank-width-ab/artifacts/task145-rerank-width-ab-suite.json`
- Results:
  `reviews/task-145/001-rerank-width-ab/artifacts/suite-results.jsonl`
- Manifest:
  `reviews/task-145/001-rerank-width-ab/artifacts/suite-manifest.json`
- Packet manifest:
  `reviews/task-145/001-rerank-width-ab/artifacts/manifest.md`

The suite used a release PG18 backend. `suite-manifest.json` records the
coordinator node as `build_profile=release` with installed backend
`/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so` and sha256
`a821e3ee67501cc7489dcc9380e2bfab867b33388f600ef1f8109d19751a5bf8`.
Every latency/recall result row carries `backend_build_profile=release` and
`backend_node_profiles=coordinator:28818:release`.

## Matrix

- 10k n128/b0
- 50k n1024/b0
- 100k n1024/b0
- nprobe: 8, 16, 32, 64, 96
- A/B: `rerank_width=0` (full rerank) vs `rerank_width=50`
- Controls held constant:
  - `ec_spire.leaf_score_only_routing=on`
  - `ec_spire.route_overfetch_multiplier=1.0`
  - `ec_spire.probe_distance_ratio=0`
  - `boundary_replica_count=0`
  - `source_identity=include`
  - storage format `rabitq`

## Headline Result

`rerank_width=50` preserved `distinct_recall@k` exactly in every measured cell
and sharply reduced pipeline latency.

At nprobe96:

| Cell | Full rerank p95 | Width 50 p95 | Speedup | Recall delta |
| --- | ---: | ---: | ---: | ---: |
| 10k n128/b0 | 285.123 ms | 10.325 ms | 27.6x | 0.0000 |
| 50k n1024/b0 | 206.638 ms | 15.059 ms | 13.7x | 0.0000 |
| 100k n1024/b0 | 403.823 ms | 20.764 ms | 19.4x | 0.0000 |

Storage is unchanged by the A/B axis because both variants use the same loaded
index:

| Cell | SPIRE index size | Index bytes/row |
| --- | ---: | ---: |
| 10k n128/b0 | 10.1 MiB | 1058.4 B |
| 50k n1024/b0 | 54.4 MiB | 1139.8 B |
| 100k n1024/b0 | 97.8 MiB | 1025.1 B |

## Decision

Promote `rerank_width=50` as the next Task 145 economy candidate. The packet
does not flip defaults; it asks for review of the release evidence before the
next implementation/config slice.

## Hygiene

The truth-cache JSON files generated during the run are ignored and not part of
the review packet. The committed evidence is the suite config, suite manifest,
structured `suite-results.jsonl`, command logs, and packet manifest.
