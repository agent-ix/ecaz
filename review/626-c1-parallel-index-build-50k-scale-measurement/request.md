# Review Request: Parallel Index Build 50k Scale Measurement

Current head: `0c0984c`

Scope:
- `review/626-c1-parallel-index-build-50k-scale-measurement/artifacts/manifest.md`
- `review/626-c1-parallel-index-build-50k-scale-measurement/artifacts/pg18_parallel_build_50k_scale_timing.sql`
- `review/626-c1-parallel-index-build-50k-scale-measurement/artifacts/pg18_parallel_build_50k_scale_timing.log`

Question:
- After the native graph-build scratch reuse from packet 625, how does the PG18
  parallel index build path scale from 10k to 50k rows?

Result:
- Fixture: 50,000 rows, 64 dimensions, default `turboquant`, no
  `build_source_column`, `m = 6`, `ef_construction = 40`.
- Serial create-index time: 53,683.587 ms.
- Parallel create-index time: 52,391.576 ms.
- Parallel was about 2.4% faster in this single 50k run.
- Both measured index sizes were identical at 11,624,448 bytes.

Phase Breakdown:
- Serial heap ingest: 8,618.142 ms.
- Parallel heap ingest total: 7,121.196 ms.
- Parallel setup: 2.697 ms.
- Parallel queue drain/finish: 412.374 ms.
- Parallel sort plus `BuildState::push`: 6,706.119 ms.
- Serial graph construction: 44,626.766 ms.
- Parallel graph construction: 44,867.640 ms.
- Graph construction accounts for about 83% of serial wall time and about 86%
  of parallel wall time.

Interpretation:
- The parallel build path still works at 50k and launches 4 workers, but the
  larger fixture narrows the wall-clock gain to about 2.4%.
- Graph construction remains dominant.
- The parallel path also exposes a second concrete bottleneck: the leader's
  sort/push phase is 6.7 seconds at 50k. Inspecting the code points to
  `BuildState::push` doing a linear duplicate scan across accumulated tuples,
  so unique-heavy builds pay O(N^2) ingest overhead. That affects both serial
  and parallel builds, but it is separately visible in the parallel phase
  counters.

Validation:
- Raw log is stored packet-locally at
  `artifacts/pg18_parallel_build_50k_scale_timing.log`.
- SQL fixture is stored packet-locally at
  `artifacts/pg18_parallel_build_50k_scale_timing.sql`.
- Artifact metadata and key lines are recorded in `artifacts/manifest.md`.
- The run used `ecaz dev sql --pg 18 --log-output`, not shell redirection or
  `script`.

Review focus:
- Whether this 50k one-round measurement is sufficient to justify optimizing
  `BuildState::push` before further parallel-ingest tuning.
- Whether a follow-up should remeasure 50k after replacing linear duplicate
  lookup with an indexed lookup.
