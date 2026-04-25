# Review Request: Native Decoded Score Workspace Measurement

Current head: `76e1b6c`

Scope:
- Code checkpoint: `76e1b6c` (`Precompute native build code score values`)
- Baseline checkpoint: `184a030` (`Add native graph scratch cache measurement packet`)
- `review/629-c1-native-decoded-score-workspace-measurement/artifacts/manifest.md`
- `review/629-c1-native-decoded-score-workspace-measurement/artifacts/pg18_native_decoded_score_workspace_10k1536_timing.sql`
- `review/629-c1-native-decoded-score-workspace-measurement/artifacts/pg18_native_decoded_score_workspace_10k1536_baseline_184a030.log`
- `review/629-c1-native-decoded-score-workspace-measurement/artifacts/pg18_native_decoded_score_workspace_10k1536_current_76e1b6c.log`

Question:
- Does predecoding no-QJL 4-bit native build codes into a bounded f32 workspace
  materially reduce code-to-code graph scoring time?

Result:
- Fixture: 10,000 rows, 1,536 dimensions, indexed `tqvector`,
  default `turboquant`, no `build_source_column`, `m = 6`,
  `ef_construction = 40`.
- The fixture uses `tqvector` intentionally. `ecvector` builds retain raw
  source vectors and score with the source-vector graph metric, so they do not
  exercise this optimization.
- Baseline serial create-index time: 304,071.708 ms.
- Current serial create-index time: 54,708.383 ms (`~82.0%` faster).
- Baseline parallel create-index time: 303,680.367 ms.
- Current parallel create-index time: 54,874.921 ms (`~81.9%` faster).
- Baseline serial graph phase: 303,529.692 ms.
- Current serial graph phase: 54,198.793 ms (`~82.1%` reduction).
- Baseline parallel graph phase: 303,183.836 ms.
- Current parallel graph phase: 54,360.998 ms (`~82.1%` reduction).
- All measured index sizes were identical at 11,739,136 bytes.

Interpretation:
- This is a real win, but only for the code-scored native graph lane. The
  earlier 64-dimensional `ecvector` fixture does not activate this path because
  `ecvector` builds score raw source vectors.
- The old 1,536-dimensional `tqvector` code-scored graph path was dominated by
  repeated packed-code nibble decoding in `score_ip_codes_lite`. Predecoding the
  tuple codes once per build removes that work from every candidate comparison.
- Parallel build remains almost identical to serial on this fixture because
  graph assembly is still leader-local serial work. The decoded workspace
  reduces the serial graph cost; it does not make graph assembly parallel.
- The workspace is bounded at 64 MiB and falls back to the old metric path when
  the build is not no-QJL 4-bit, code lengths do not match, or the decoded
  values would exceed the cap.

Validation:
- Code checkpoint gates run before commit:
  - `cargo test`
  - `cargo pgrx test pg18`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `git diff --check`
- Measurement commands used `ecaz dev sql --pg 18 --log-output`, not shell
  redirection or `script`.
- Raw logs are stored packet-locally in `artifacts/`.
- Artifact metadata and key result lines are recorded in
  `artifacts/manifest.md`.

Review focus:
- Whether the workspace activation conditions and 64 MiB cap are the right
  safety boundary.
- Whether this closes the repeated no-QJL 4-bit decode bottleneck for native
  code-scored graph builds.
- Whether the next implementation slice should move back to 50k `ecvector`
  source-scored graph work or start a design packet for actual parallel graph
  assembly.
