# Task 144 Packet 001 Artifact Manifest

- Head SHA: `fc8544288cf992f1742132c861cb8b7b0afbcc5f`
- Branch: `task-144-spire-closure-ratio-pruning`
- Task bucket: `reviews/task-144/001-geometry-diagnostics`
- Slice: Phase 0 diagnostic substrate for Task 144 geometry measurements.
- Lane / fixture / storage / rerank mode: code-only CLI/suite instrumentation; no corpus fixture was loaded in this packet.
- Isolated/shared surface: not applicable for this code slice.
- Backend profile: not applicable for unit tests; benchmark use must still run release `ecaz bench suite`.

## Artifacts

| Artifact | Command | Timestamp | Key result |
| --- | --- | --- | --- |
| `artifacts/cargo-test-ecaz-cli-spire-pipeline.log` | `cargo test -p ecaz-cli spire_pipeline --no-default-features` | 2026-07-05 | `30 passed; 0 failed; 0 ignored; 409 filtered out`. |

## Code Surface

- `crates/ecaz-cli/src/commands/bench/spire_pipeline.rs`
  - Adds `--geometry-output`.
  - Writes JSONL rows for active leaf-size distribution:
    `spire_geometry_leaf_size_summary`.
  - Writes per-query true-neighbor list concentration for the active
    single-assignment surface:
    `spire_geometry_true_neighbor_concentration` with
    `mode="single_assignment"`.
  - Requires `--include-recall`, reuses the existing exact-truth path, and
    reads SQL-visible `ec_spire_index_leaf_snapshot` /
    `ec_spire_index_leaf_target_assignment_snapshot` surfaces.
- `crates/ecaz-cli/src/commands/bench/suite.rs`
  - Adds `geometry_output` to `spire-pipeline` suite steps.
  - Supports `${artifact_dir}` rewriting, expected-artifact tracking, and CLI
    argument expansion.

## Scope Notes

- This packet intentionally does not claim Task 144 closeout.
- It covers the Phase 0 single-assignment diagnostic substrate needed to measure
  "how many lists hold the true top-k" and leaf-size variance under current
  SPIRE assignment.
- Closure-simulated concentration and the closure/pruning reloption/GUC remain
  follow-on Task 144 slices.
- No benchmark matrix was run here; all future Task 144 measurements must use
  `ecaz bench suite` with `geometry_output` checked into the owning packet.

## Feedback Scan

- Latest Task 141 feedback remains `reviews/task-141/001-release-anchor-rebaseline/feedback/2026-07-05-02-agent-ix.md`, approving the P0 substrate and unblocking Tasks 142-146.
- No Task 143 or Task 144 feedback files existed before this packet.
