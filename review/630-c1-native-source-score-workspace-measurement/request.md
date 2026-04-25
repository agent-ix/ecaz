# Review Request: Native Source Score Workspace Measurement

Current head: `7f2d03e`

Scope:
- Code checkpoint: `7f2d03e` (`Flatten native build source score values`)
- `review/630-c1-native-source-score-workspace-measurement/artifacts/manifest.md`
- `review/630-c1-native-source-score-workspace-measurement/artifacts/pg18_native_source_score_workspace_50k_timing.sql`
- `review/630-c1-native-source-score-workspace-measurement/artifacts/pg18_native_source_score_workspace_50k_timing.log`

Question:
- Does flattening native source-vector scoring input into a bounded contiguous
  workspace materially reduce the remaining serial graph phase on the
  source-scored `ecvector` build fixture?

Result:
- Fixture: 50,000 rows, 64 dimensions, default `turboquant`, no
  `build_source_column`, `m = 6`, `ef_construction = 40`.
- Serial create-index time: 29,692.277 ms.
- Parallel create-index time: 28,794.693 ms.
- The parallel path launched 4 workers and was now about 3.0% faster than
  serial in this single run, but both paths still spend roughly 27.8 seconds in
  serial graph construction.
- Both measured index sizes were identical at 11,624,448 bytes.

Comparison to packet 628:
- Serial create-index time improved from 30,333.870 ms to 29,692.277 ms
  (`~2.1%` faster).
- Parallel create-index time improved from 30,845.279 ms to 28,794.693 ms
  (`~6.6%` faster).
- Serial graph construction fell from 28,556.153 ms to 27,886.740 ms.
- Parallel graph construction fell from 29,816.052 ms to 27,811.377 ms.
- Parallel sort plus `BuildState::push` stayed small: 152.194 ms.

Interpretation:
- The source-workspace change is a smaller but real hot-path cleanup for
  `ecvector` source scoring. It removes per-candidate source payload decoding
  overhead where the bounded workspace can hold the flattened vectors.
- This result is intentionally separate from packet 629: indexed `tqvector`
  code scoring saw a much larger win from decoded code-value precomputation,
  while this packet measures the raw-source metric path used by `ecvector`.
- The remaining bottleneck is still serial HNSW graph assembly. The next useful
  work should target graph assembly itself rather than more coordinator
  threshold tuning.

Validation:
- Code checkpoint gates run before commit:
  - `cargo test`
  - `cargo pgrx test pg18`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `git diff --check`
- Raw log is stored packet-locally at
  `artifacts/pg18_native_source_score_workspace_50k_timing.log`.
- SQL fixture is stored packet-locally at
  `artifacts/pg18_native_source_score_workspace_50k_timing.sql`.
- Artifact metadata and key lines are recorded in `artifacts/manifest.md`.
- The run used `ecaz dev sql --pg 18 --log-output`, not shell redirection or
  `script`.

Review focus:
- Whether this closes the source-scoring workspace slice.
- Whether the next implementation should move to a design/implementation slice
  for parallel graph assembly or continue reducing individual serial graph
  hot spots.
