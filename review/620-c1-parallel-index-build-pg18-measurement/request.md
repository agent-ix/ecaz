# Review Request: Parallel Index Build PG18 Measurement

Current head: `4305d77`

Scope:
- `review/620-c1-parallel-index-build-pg18-measurement/artifacts/manifest.md`
- `review/620-c1-parallel-index-build-pg18-measurement/artifacts/pg18_parallel_build_timing.sql`
- `review/620-c1-parallel-index-build-pg18-measurement/artifacts/pg18_parallel_build_timing.log`

Question:
- Does the first executable parallel index build path produce a speedup on a
  small PG18 fixture, or is the cost still dominated by leader-side work?

Result:
- It does not produce a speedup on this fixture.
- Fixture: 10,000 rows, 64 dimensions, default `turboquant`, no
  `build_source_column`, `m = 6`, `ef_construction = 40`.
- Serial create-index times:
  - round 1: 9139.351 ms
  - round 2: 9011.378 ms
- Parallel create-index times:
  - round 1: 9633.868 ms
  - round 2: 9350.732 ms
- Average serial: 9075.365 ms
- Average parallel: 9492.300 ms
- Parallel was about 4.6% slower on this warm-cache fixture.
- All measured index sizes were identical at 2,334,720 bytes.

Interpretation:
- This matches the implementation shape from packet 619: workers parallelize
  heap scan and tuple encoding, but the leader still drains/merges tuples and
  performs serial HNSW graph/page assembly.
- On this fixture, parallelizing ingestion alone is not enough to overcome
  worker launch, DSM, queue transport, and leader merge overhead.
- This is useful evidence against threshold tuning as the next step. The next
  implementation work should isolate the remaining build phases before claiming
  speedup: worker encode time, leader queue drain/decode, leader `BuildState`
  ingestion, and graph/page flush.

Validation:
- Raw log is stored packet-locally at
  `artifacts/pg18_parallel_build_timing.log`.
- SQL fixture is stored packet-locally at
  `artifacts/pg18_parallel_build_timing.sql`.
- Artifact metadata and key lines are recorded in `artifacts/manifest.md`.

Review focus:
- Whether the fixture is sufficient as an initial no-speedup finding for the
  first executable coordinator.
- Whether the next implementation slice should add explicit build phase timing
  before attempting leader participation or shared tuple sorting.
