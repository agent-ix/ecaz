# Review Request: Parallel Index Build Phase Measurement

Current head: `d5e404b`

Scope:
- `review/622-c1-parallel-index-build-phase-measurement/artifacts/manifest.md`
- `review/622-c1-parallel-index-build-phase-measurement/artifacts/pg18_parallel_build_phase_timing.sql`
- `review/622-c1-parallel-index-build-phase-measurement/artifacts/pg18_parallel_build_phase_timing.log`

Question:
- With the phase timing surface from packet 621, where does PG18 build time go
  for the first executable parallel index build path?

Result:
- Fixture: 10,000 rows, 64 dimensions, default `turboquant`, no
  `build_source_column`, `m = 6`, `ef_construction = 40`.
- Serial create-index times:
  - round 1: 9033.476 ms
  - round 2: 8992.727 ms
- Parallel create-index times:
  - round 1: 8676.363 ms
  - round 2: 8712.693 ms
- Average serial: 9013.102 ms.
- Average parallel: 8694.528 ms.
- Parallel was about 3.5% faster in this run.
- All measured index sizes were identical at 2,334,720 bytes.

Phase Breakdown:
- Average serial heap ingest: 537.568 ms.
- Average parallel heap ingest total: 397.861 ms.
- Average parallel setup: 2.780 ms.
- Average parallel queue drain/finish: 138.543 ms.
- Average parallel sort plus `BuildState::push`: 256.521 ms.
- Average serial graph construction: 8384.270 ms.
- Average parallel graph construction: 8214.275 ms.
- Graph construction accounts for about 93% of serial create-index wall time
  and about 94% of parallel create-index wall time.
- Staging and page writes are small by comparison: roughly 48-53 ms staging
  and 18-20 ms writes per round.

Interpretation:
- The worker path is functional and can reduce heap ingestion time on this
  fixture.
- The current implementation is still dominated by serial HNSW graph
  construction. Transport is visible but not the main cost: drain plus
  sort/push was roughly 395 ms total across the two parallel rounds, while graph
  construction was roughly 8.2 seconds.
- This points away from threshold tuning and toward either graph construction
  optimization or a more substantial build algorithm change. Leader
  participation in heap scan is unlikely to matter much while graph construction
  remains this dominant.

Validation:
- Raw log is stored packet-locally at
  `artifacts/pg18_parallel_build_phase_timing.log`.
- SQL fixture is stored packet-locally at
  `artifacts/pg18_parallel_build_phase_timing.sql`.
- Artifact metadata and key lines are recorded in `artifacts/manifest.md`.
- The run used `ecaz dev sql --log-output`, not shell redirection or `script`.
- Measurement artifact head is `4692296`; the subsequent `d5e404b` CLI commit
  only made dev tooling more PG-version-aware and did not change extension
  build behavior.

Review focus:
- Whether this phase breakdown is sufficient to steer the next implementation
  slice toward graph construction.
- Whether we need a larger fixture before deciding not to prioritize leader
  participation or queue transport.
