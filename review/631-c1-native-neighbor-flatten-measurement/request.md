# Review Request: Native Neighbor Flatten Measurement

Current head: `6815332`

Scope:
- Code checkpoint: `6815332` (`Avoid hash set allocation when flattening build neighbors`)
- `src/am/ec_hnsw/build.rs`
- `review/631-c1-native-neighbor-flatten-measurement/artifacts/manifest.md`
- `review/631-c1-native-neighbor-flatten-measurement/artifacts/pg18_native_neighbor_flatten_50k_timing.sql`
- `review/631-c1-native-neighbor-flatten-measurement/artifacts/pg18_native_neighbor_flatten_50k_timing.log`

Question:
- Does replacing per-node `HashSet` allocation in
  `flatten_native_neighbor_slots` with small-vector linear dedupe reduce the
  remaining graph build cost without changing neighbor ordering semantics?

Result:
- Fixture: 50,000 rows, 64 dimensions, default `turboquant`, no
  `build_source_column`, `m = 6`, `ef_construction = 40`.
- Serial create-index time: 29,432.464 ms.
- Parallel create-index time: 28,119.478 ms.
- The parallel path launched 4 workers and was about 4.5% faster than serial in
  this single run, but graph construction is still the dominant cost.
- Both measured index sizes were identical at 11,624,448 bytes.

Comparison to packet 630:
- Serial create-index time improved from 29,692.277 ms to 29,432.464 ms
  (`~0.9%` faster).
- Parallel create-index time improved from 28,794.693 ms to 28,119.478 ms
  (`~2.3%` faster).
- Serial graph construction fell from 27,886.740 ms to 27,560.080 ms.
- Parallel graph construction fell from 27,811.377 ms to 27,029.382 ms.
- Parallel sort plus `BuildState::push` stayed small: 174.548 ms.

Interpretation:
- This is a small allocation cleanup, not the main path to scalable parallel
  build. It removes one obvious per-node hash table from the graph finalize
  path and preserves first-seen layer order, self-link filtering, and dedupe.
- The measurement moved in the expected direction, but the residual graph
  phase is still roughly 27 seconds on this fixture. Further meaningful
  progress needs either a true graph-assembly parallelization design or a
  larger serial HNSW search reduction.

Validation:
- Code checkpoint gates run before commit:
  - `cargo test`
  - `cargo pgrx test pg18`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `git diff --check`
- Raw log is stored packet-locally at
  `artifacts/pg18_native_neighbor_flatten_50k_timing.log`.
- SQL fixture is stored packet-locally at
  `artifacts/pg18_native_neighbor_flatten_50k_timing.sql`.
- Artifact metadata and key lines are recorded in `artifacts/manifest.md`.
- The run used `ecaz dev sql --pg 18 --log-output`, not shell redirection or
  `script`.

Review focus:
- Whether the small-vector dedupe is an acceptable replacement for the
  per-node `HashSet` in native graph neighbor flattening.
- Whether this closes the low-risk serial cleanup lane so the next packet can
  focus on a graph-assembly parallelization design.
