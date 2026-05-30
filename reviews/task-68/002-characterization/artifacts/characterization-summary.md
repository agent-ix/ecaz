# Task 68 SPIRE Build Characterization Summary

## Suite Scope

- Suite config: `reviews/task-68/002-characterization/artifacts/suite.json`
- Runner: `ecaz bench suite`
- Database: `task68_spire_char`
- PostgreSQL: 18.3, Homebrew, aarch64 Apple Darwin
- Fixture storage format: `turboquant`
- Surface: one index per table for the measured `CREATE INDEX` steps

The suite first loaded the 10k and 100k M5 DBpedia fixtures, then dropped the
loader-created index and rebuilt a single measured SPIRE index per corpus with
structured `ec_spire_ambuild_timing` notices.

## AM Timing Split

| fixture | rows | nlists | fanout | total_ms | heap_scan_ms | sample_collect_ms | kmeans_ms | kmeans_calls | assignment_ms | recursive_kmeans_ms | recursive_kmeans_calls | recursive_kmeans_max_level | recursive_assignment_ms | draft_ms | top_graph_ms | pq4_training_ms | object_store_ms | publish_ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 10000 | 32 | 8 | 806 | 119 | 0 | 147 | 1 | 15 | 0 | 1 | 1 | 0 | 499 | 24 | 0 | 24 | 0 |
| 100k | 100000 | 128 | 8 | 21814 | 1220 | 0 | 486 | 1 | 570 | 2 | 1 | 1 | 0 | 19282 | 252 | 0 | 252 | 1 |

## K-Means Rollup

| fixture | top-level calls | top-level total_ms | top-level mean_ms | recursive calls | recursive total_ms | recursive mean_ms | combined calls | combined total_ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 1 | 147 | 147 | 1 | 0 | 0 | 2 | 147 |
| 100k | 1 | 486 | 486 | 1 | 2 | 2 | 2 | 488 |

## Loader Wall Time

| fixture | loader index build | loader prefix total |
| --- | ---: | ---: |
| 10k | 805.46 ms | 3.39 s |
| 100k | 21.80 s | 46.26 s |

## Static Call Audit

Packet 001 captured the build-path common-training call sites in
`reviews/task-68/001-spire-build-timing-notices/artifacts/common-training-call-audit.txt`.

Build-path users:

- `src/am/ec_spire/build/training.rs`: single-level/relation-build k-means,
  batch assignment, normalization, deterministic sample selection.
- `src/am/ec_spire/build/recursive.rs`: recursive per-level k-means and batch
  assignment.

Non-build SPIRE users:

- `src/am/ec_spire/update/materialization.rs`: split replacement materialization
  k-means.
- `src/am/ec_spire/update/routing.rs`: scheduled merge centroid normalization.

## Ranked P0 Slices

1. Draft assembly/object construction: 19.282s of the 21.814s measured 100k build, and 499ms of the 806ms measured 10k build. This dominates at both sizes and is the first P0 target.
2. Heap scan plus tuple/vector collection: 1.220s at 100k and 119ms at 10k. It is the next largest self-contained phase and scales directly with input rows.
3. Top-level assignment: 570ms at 100k and 15ms at 10k. This is row-parallel work and now has a shared batch helper from Task 69.
4. Shared k-means training: 488ms combined at 100k and 147ms at 10k. This is no longer the largest phase after Task 69, but remains worth keeping in the measurement gate.
5. Top-graph/object-store publish: 252ms at 100k and 24ms at 10k. This is visible but not the first optimization target in this lane.

PQ4 training is not ranked for this build path because the measured SPIRE
build reported `pq4_training_ms=0`; current SPIRE build defers grouped-PQ
model work outside this path.
