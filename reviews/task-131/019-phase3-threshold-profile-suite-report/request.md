# Task 131 Review Request: Phase 3 Threshold Profile Suite Report

Head: `be234f72f1dc5e8eca437595202759b992944ea6`

## Scope

This checkpoint wires packet 018's candidate-derived threshold diagnostic into the standard SPIRE pipeline benchmark report path:

- under `--include-production-read-profile`, `ecaz bench spire-pipeline` now also calls `ec_spire_remote_search_production_candidate_threshold_profile(...)`
- aggregates rows by `nprobe` and `node_id`
- renders a `Production candidate-derived threshold profile` section with:
  - derived threshold score/IP min/max
  - selected/evaluated PID sums
  - sound-bound availability/missing sums
  - threshold block selected/skipped sums
  - threshold row selected/skipped sums
  - leaf summary scoring nanos

This still does not claim a latency win or implement scan-time early stop. It gives the approved suite runner a durable place to record whether real production candidate frontiers would make any selected worker blocks/rows skippable.

## Validation

Artifacts are listed in `artifacts/manifest.md`.

- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli spire_pipeline_renders_production_threshold_profile`
- `cargo test -p ecaz-cli spire_pipeline_sql_uses_public_snapshot_contracts`

All passed. `cargo check -p ecaz-cli` emitted an existing unrelated dead-code warning for `LoadedDistributedPlacementConfig::path`.

## Next Work

Run the local multi-instance suite with `--include-production-read-profile` to collect real candidate-derived threshold boundability rows, then use those numbers to decide whether the Phase 3 early-stop prototype is worth implementing or should be shelved.
