# Review Request: Phase 13e Local Production Read Gates

## Summary

This packet closes the local-vs-AWS production read gap called out in packet 985 feedback. The production `ecaz bench suite` path now reproduces the formerly AWS-only `custom_scan_tuple_delivery` failure locally, then passes locally after the fix.

Code checkpoint: `908be140cc4734521913c3c7b282747e67262e67`

## What Changed

- Added `scripts/run_spire_phase13e_local_gates_pg18.sh` so repeated local SPIRE core/extended gates are driven by one repo-owned script.
- Added `ecaz bench spire-pipeline --production-read-only` and suite JSON support for `production_read_only`.
- In production-read-only mode, `spire-pipeline` skips local heap diagnostic snapshots and forces the KNN query onto the SPIRE CustomScan tuple delivery path with `enable_indexscan = off`.
- Updated the Phase 13e static remote placement fixture to:
  - run the production `ecaz bench suite` step locally,
  - keep coordinator and remote shard schemas fingerprint-compatible,
  - derive remote selected PIDs from exported assignment files after remote placement publication,
  - use production placement publishing instead of test-only placement rewrites in the insert/read fixture.
- Fixed coordinator insert classification after remote placement publication by using the coordinator fanout anchor rather than the local active epoch anchor.
- Kept AWS harness fixes staged in code, but no AWS validation was run for this packet.

## Evidence

Packet-local manifest: `reviews/task-30/987-spire-phase13e-local-gates/artifacts/manifest.md`

Core local suite:

- `phase13e-static-remote-placement`: pass
- `multicluster-customscan-read`: pass
- `insert-read-after-customscan-helper`: pass
- `insert-read-after-customscan-trigger`: pass
- `transport-overlap`: pass

Production suite evidence from `spire-pipeline.log`:

- `production_read_only: true`
- tuple transport ready with `pg_binary_attr_v1`
- recall@k `1.0000`
- production read profile status `ready`
- `remote_pid_sum = 3`
- `dispatch_sum = 3`
- `socket_open_sum = 3`
- `candidate_query_sum = 3`
- `heap_query_sum = 3`
- `returned_sum = 6`

Extended local suite:

- All 17 Stage E predispatch, candidate, network, transport, and lifecycle fault gates passed.

Focused tests:

- `cargo test --package ecaz-cli spire_pipeline`: 19 passed
- `cargo test --package ecaz-cli render_spire_registrations`: 7 passed

## Notes

No AWS instances were started or used for this packet. AWS should remain paused until this packet is reviewed and accepted as the local gate baseline.
