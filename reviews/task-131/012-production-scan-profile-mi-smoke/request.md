# Review Request: Production Scan Profile Multi-Instance Smoke

- task: 131
- packet: `reviews/task-131/012-production-scan-profile-mi-smoke/`
- code commit under review: `7425bf051bb3db157c7ab62de4e738754f0adc56`
- predecessor packet: `reviews/task-131/011-phase0-production-scan-profile/`

## Context

Packet 011 exposed production selected-leaf scan profiles but only had build/unit validation. Its request explicitly called out that local multi-instance execution evidence was still missing. This packet supplies that evidence and fixes the runtime decoder issue found by the smoke.

This remains Phase 0 instrumentation work. It does not implement Phase 2 barrier removal or Phase 3 streaming global-threshold feedback.

## Changes

- Fixed the CLI production scan-profile decoder to read `node_id` as `i64`, matching the SQL `bigint` returned by `ec_spire_remote_search_production_scan_profile`.
- Re-ran the local four-instance PG18 static remote placement harness with `ecaz bench suite` and `ecaz bench spire-pipeline --include-production-read-profile`.

## Evidence

See `artifacts/manifest.md`.

- The local multi-instance harness passed.
- The generated suite step succeeded and wrote structured `results.jsonl`.
- The CLI report includes `Production selected-leaf scan profile`.
- Structured scan-profile rows were emitted for remote nodes 2, 3, and 4.
- The smoke showed `sound_bound_available_sum=0` and `sound_bound_missing_sum=1` for each selected remote PID in this fixture, which means this current storage/fixture shape does not yet expose a sound early-stop bound for Phase 3.

## Validation

- `cargo build -p ecaz-cli` passed with an existing dead-code warning.
- `cargo test -p ecaz-cli spire_pipeline_renders_production_scan_profile` passed.
- `cargo test -p ecaz-cli spire_pipeline_sql_uses_public_snapshot_contracts` passed.
- Local four-instance PG18 harness passed via `scripts/run_spire_phase13e_static_remote_placement_pg18.sh`.

## Reviewer Notes

- This is intentionally a smoke, not a Task 131 performance matrix.
- The important result is that the production scan-time profile now survives real local multi-instance fanout and records per-worker scan counters in suite output.
- The next implementation work should move to Phase 2: remove the candidate-phase barrier where safe, while preserving strict/degraded semantics and cleanup behavior.
