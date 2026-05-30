# Review Request: Task 68 Packet 002 SPIRE Build Characterization

Code baseline: `c580065eacec32170931e0a44be5b041f09229cd`

## Summary

This packet completes Task 68 Phase 1 characterization for the local M5 SPIRE build lane. It uses a checked-in `ecaz bench suite` config to load the 10k and 100k DBpedia fixtures, then measures one `ec_spire` index per table through packet-local `CREATE INDEX` logs that capture the structured `ec_spire_ambuild_timing` notice from Packet 001.

Artifact manifest: `artifacts/manifest.md`

Suite config: `artifacts/suite.json`

Suite status:

```text
completed=7 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Measurement Context

- Database: `task68_spire_char`
- PostgreSQL: 18.3 on local M5/Homebrew
- Extension: `ecaz 0.1.1`, AM list includes `ec_spire`
- 10k fixture: `data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_*`
- 100k fixture: `data/task60_m5_dbpedia_staged/ec_real_100k_*`
- Build reloptions:
  - 10k: `nlists=32`, `recursive_fanout=8`, `nprobe=24`, `rerank_width=25`, `storage_format='turboquant'`, `boundary_replica_count=0`, top graph enabled
  - 100k: `nlists=128`, `recursive_fanout=8`, `nprobe=24`, `rerank_width=25`, `storage_format='turboquant'`, `boundary_replica_count=0`, top graph enabled

## Key Results

Measured SPIRE `CREATE INDEX` split:

| fixture | total_ms | heap_scan_ms | sample_collect_ms | kmeans_ms | assignment_ms | recursive_kmeans_ms | draft_ms | top_graph_ms | pq4_training_ms | object_store_ms | publish_ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 806 | 119 | 0 | 147 | 15 | 0 | 499 | 24 | 0 | 24 | 0 |
| 100k | 21814 | 1220 | 0 | 486 | 570 | 2 | 19282 | 252 | 0 | 252 | 1 |

Per-level k-means rollup:

| fixture | top-level calls | top-level total_ms | recursive calls | recursive total_ms | recursive max level | combined total_ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 1 | 147 | 1 | 0 | 1 | 147 |
| 100k | 1 | 486 | 1 | 2 | 1 | 488 |

Loader wall time for the same fixture profiles:

| fixture | loader index build | loader prefix total |
| --- | ---: | ---: |
| 10k | 805.46 ms | 3.39 s |
| 100k | 21.80 s | 46.26 s |

Raw notices:

- `artifacts/create-10k-spire-profile-index.log`
- `artifacts/create-100k-spire-profile-index.log`

## Static Call Audit

Packet 001 provides the static `common_training::*` audit at:

`reviews/task-68/001-spire-build-timing-notices/artifacts/common-training-call-audit.txt`

Build-path consumers are:

- `src/am/ec_spire/build/training.rs`: top-level k-means, assignment, normalization, deterministic sample selection.
- `src/am/ec_spire/build/recursive.rs`: recursive per-level k-means and assignment.

## Ranked P0 Recommendations

1. Draft assembly/object construction. This dominates the 100k measured build: `draft_ms=19282` out of `total_ms=21814`, and also dominates the 10k build at `499/806ms`.
2. Heap scan plus tuple/vector collection. This is `1220ms` at 100k and `119ms` at 10k, and scales with input rows.
3. Top-level assignment. This is `570ms` at 100k and now has a shared batch assignment helper from Task 69.
4. Shared k-means training. It is no longer the top cost after Task 69, but remains visible at `488ms` combined on 100k.
5. Top-graph/object-store publish. Visible at `252ms` on 100k, but lower priority than draft assembly and heap/assignment work.

PQ4 training is not a P0 build target in this packet because measured SPIRE build reports `pq4_training_ms=0`; current grouped-PQ work is outside this build path.

## Reviewer Ask

Please verify the Phase 1 ranking and whether the first P0 implementation slice should target draft assembly/object construction before heap scan and assignment.
