# Task 144 Packet 002 Artifact Manifest

- Head SHA: `7c2cf2eb38974cae0dead89b0ae7e256c545320f`
- Branch: `task-144-spire-closure-ratio-pruning`
- Task bucket: `reviews/task-144/002-closure-geometry-simulation`
- Slice: Phase 0 closure-simulated geometry diagnostics for Task 144.
- Lane / fixture / storage / rerank mode: code-only CLI/suite instrumentation; no corpus fixture was loaded in this packet.
- Isolated/shared surface: not applicable for this code slice.
- Backend profile: not applicable for unit tests; future geometry measurements must run release `ecaz bench suite`.

## Artifacts

| Artifact | Command | Timestamp | Key result |
| --- | --- | --- | --- |
| `artifacts/cargo-test-ecaz-cli-spire-pipeline.log` | `cargo test -p ecaz-cli spire_pipeline --no-default-features` | 2026-07-05 | `30 passed; 0 failed; 0 ignored; 409 filtered out`. |

## Code Surface

- `crates/ecaz-cli/src/commands/bench/spire_pipeline.rs`
  - Adds `--geometry-closure-epsilon`.
  - When paired with `--geometry-output`, emits additional
    `spire_geometry_true_neighbor_concentration` rows with
    `mode="closure_simulated_ip_distance_ratio"` and `closure_epsilon=<value>`.
  - Fetches exact truth source vectors from the loaded corpus table and active
    leaf centroids from `ec_spire_index_routing_centroid_snapshot`.
  - Uses a diagnostic IP-distance proxy: `max(0, 1 - dot(vector, centroid))`;
    includes leaves whose distance is within `(1 + epsilon)` of the nearest
    centroid distance.
- `crates/ecaz-cli/src/commands/bench/suite.rs`
  - Adds `geometry_closure_epsilon` to `spire-pipeline` suite steps and expands
    it to `--geometry-closure-epsilon`.

## Scope Notes

- This packet is still Phase 0 diagnostic substrate, not closure assignment.
- The simulation does not change build assignment, routing, scan behavior, or
  recall results.
- The artifact shape now supports comparing current single-assignment
  concentration with closure-simulated concentration for pre-registered epsilon
  values in later release `ecaz bench suite` runs.

## Feedback Scan

- Latest Task 141 feedback remains `reviews/task-141/001-release-anchor-rebaseline/feedback/2026-07-05-02-agent-ix.md`, approving the P0 substrate and unblocking Tasks 142-146.
- No Task 143 or Task 144 feedback files existed before this packet.
