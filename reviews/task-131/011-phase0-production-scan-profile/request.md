# Review Request: Production Selected-Leaf Scan Profile

- task: 131
- packet: `reviews/task-131/011-phase0-production-scan-profile/`
- code commit under review: `7a76413f50b49dcfd356cc8d7e7129f86b4778ca`
- predecessor packet: `reviews/task-131/010-phase0-selected-leaf-scan-profile/`

## Context

Reviewer feedback in packet 009 directed the coder path away from heap-side Phase 0/1 expansion and toward scan-time instrumentation: local kth, selected/scanned PID counters, and sound upper-bound visibility. Packet 010 added the local selected-leaf scan collector and coordinator-local SQL endpoint. This packet wires that scan-time profile through production fanout and the CLI report surface.

This is instrumentation only. It does not implement barrier removal or streaming threshold updates.

## Changes

- Added `ec_spire_remote_search_production_scan_profile(index_oid, query, top_k)`, returning one scan-profile row per participating local/remote worker.
- Added a libpq remote scan-profile dispatch query that calls the packet-010 coordinator-local scan-profile endpoint on each remote worker.
- Decoded remote scan-profile rows with epoch validation and node stamping, preserving per-worker `local_kth_score`.
- Extended `ecaz bench spire-pipeline --include-production-read-profile` to collect and render a `Production selected-leaf scan profile` table.
- The new CLI table reports scan-time counters needed for the streaming top-k path: selected/scanned PIDs, candidate rows, leaf block availability/selection/skips, sound upper-bound availability/missing counts, scoring nanos, and local kth count/min/max.

## Validation

See `artifacts/manifest.md`.

- `cargo check --lib` passed.
- `cargo test -p ecaz-cli spire_pipeline_renders_production_scan_profile` passed.
- `cargo test -p ecaz-cli spire_pipeline_sql_uses_public_snapshot_contracts` passed.
- `cargo test --lib collect_quantized_selected_leaf_scan_profile_reports_scan_counters` passed.

## Reviewer Notes

- The production read path itself is not changed to use this data. The new endpoint is a separate diagnostic surface so it does not extend the heap-side merge-before-heap path called out in packet-009 feedback.
- This packet still needs real multi-instance execution evidence before it can support any performance conclusion. The intended next use is to run `ecaz bench spire-pipeline --include-production-read-profile` in the local multi-instance setup and inspect whether local kth and sound upper-bound availability are usable for Phase 2/3.
