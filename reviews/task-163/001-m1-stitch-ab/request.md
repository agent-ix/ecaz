# Review request — Task 163 M1: sharded build + stitch (TC-039 A/B)

**Branch:** `task-163-ec-distann-m1`
**Code SHA under test:** `a375d56dd70f364f8c2389201e5524e578f0ff14`
**Milestone:** M1 (FR-077 sharded closure-overlap build + stitch)

## What landed

New pure module `src/am/ec_distann/shard_build.rs` replaces the monolithic
Vamana graph-construction core with a sharded closure-overlap build + streaming
stitch, selected by the new `build_shards` reloption (0=auto, 1=monolithic
fallback [default], ≥2 sharded). Output is a global-space `VamanaGraph`
identical in shape to the monolithic build, so record/directory/head-sample
staging in `ambuild.rs` is unchanged.

- **Closure-overlap shard assignment:** spherical k-means (shared
  `train_spherical_kmeans`) + a fresh `closure_epsilon` ε-band (the Task-144
  distance-ratio machinery is on an unmerged branch, so the band is implemented
  fresh per the task brief).
- **Per-shard Vamana:** independent, seed-deterministic, built in parallel
  (rayon), remapped to the global id space.
- **Streaming stitch:** k-way merge by global node id (ADR-085 D8 — one node
  group + prune scratch held at a time); single-shard nodes pass through
  unchanged (idempotence); multi-shard unions `robust_prune`d to `graph_degree`.
- **Reachability repair:** bounded, monotone FR-077-CON-3 guard.

## Tests

- **TC-038 proptest suite** (`shard_build::tests`, 7 green): degree ≤ R, vec_id
  uniqueness + valid edges, medoid reachability, determinism, stitch
  idempotence (FR-077-AC-2), closure duplication scaling.
- **pg_test** (`src/tests/ec_distann_basic.rs`): sharded self-recall,
  sharded determinism across reindex.

## Evidence (this packet)

TC-039 A/B: stitched (`build_shards=4`, ε=0.1) vs monolithic at 10k/50k/100k,
release-verified. See `artifacts/manifest.md` for the full tables. Headline:

- **Storage & latency identical** mono↔stitch (one record per vec_id, CON-2).
- **Build 1.5–1.7× faster** stitched at ε=0.1 (parallel shards).
- **Recall at ε=0.1 regresses** vs monolithic, gap growing with scale (10k≈0 →
  100k −0.004 at ef=200, CI-separated at ef≤32). **FR-077-AC-1's 0.001 bar is
  not met at the M0 default ε=0.1.**

## Remediation → packet 002

The closure-band sweep (`reviews/task-163/002-m1-closure-sweep/`) shows the
regression **closes at `closure_epsilon=0.3`** (stitch matches/exceeds
monolithic across the operational recall band at 50k/100k). The default is
bumped 0.1 → 0.3 on this branch. See that packet's `request.md` for the M1
parity verdict.

## Ask

Please review the stitch implementation (correctness of the union/prune,
determinism, the D8 streaming bound, and the reachability guard) and the A/B
methodology. The parity verdict itself is in packet 002.

## Notes

- Do not close this request; leaving it open per workflow.
- `build_shards` default is 1 (monolithic), so the single-node default is
  unchanged from M0 — sharding is opt-in until promoted.
