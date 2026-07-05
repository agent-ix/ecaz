# Review Request: Task 121 Phase 0 Native Local Multinode Lane

## Scope

This packet requests review for commit `69a644d9091e678ea1e58fca7dad461eb79103bc`.

It addresses the remaining actionable finding from `reviews/task-121/002-phase0-local-multinode-suite-lane/feedback/2026-06-22-01-reviewer.md`: the local multinode path no longer shells out to `scripts/run_spire_phase13e_aws_harness_local_pg18.sh`. The `ecaz dev spire-multicluster local-multinode-pg18` command now performs the local four-instance PG18 orchestration natively.

This is local-only evidence. No AWS resources were used.

## Code Changes

- Replaced the local multinode bash wrapper path with native Rust orchestration.
- The native path initializes and starts four separate local PG18 instances on one machine:
  - coordinator
  - remote node 2
  - remote node 3
  - remote node 4
- The coordinator postmaster receives the remote conninfo provider env vars before startup.
- The correctness fixture now generates a 10k corpus locally, loads the coordinator, exports coordinator leaf-owned assignments, splits remote corpora, loads each remote, materializes remote leaf ownership, publishes placements, registers remote descriptors, and runs distributed read smoke checks.
- `smoke-customscan-read.sql` now accepts an explicit `index_name` psql variable so matrix cells can use non-default index names.
- The `spire-local-multinode` suite step expected topology artifact now points at `target/spire-local-multinode-{run_id}/topology.local.json`, matching the native path.

## Evidence

See `artifacts/manifest.md`.

- `cargo test -p ecaz-cli commands::bench::suite`: 54 passed.
- `cargo build -p ecaz-cli --bin ecaz`: passed with the pre-existing `LoadedDistributedPlacementConfig::path` dead-code warning.
- Dry-run SuiteConfig shows `ecaz bench suite` still emits a `dev spire-multicluster local-multinode-pg18` step with matrix controls and the new native topology artifact path.
- Live local multinode smoke passed with four separate local PG18 instances:
  - coordinator port `39920`
  - remote ports `39921`, `39922`, `39923`
- Distributed read smoke observed `Custom Scan (EcSpireDistributedScan)`, `remote_fanout: 3`, and `tuple_transport_status: ready`.
- Production read profile returned `status ready`, `result_source remote_heap_candidates`, `dispatch_count 3`, and zero timeout/cancel/degraded skips.

## Artifact Hygiene

The live smoke generated large shard TSVs and coordinator assignment TSVs under the packet artifact tree. Those TSV files are intentionally not part of this packet commit; the manifest records row counts and cites the compact logs/JSON evidence instead.

## Residual Work

This packet closes the native local multinode harness-replacement issue for Phase 0. It does not run the Phase 2 factorial benchmark matrix; that remains the next local-only Task 121 step after this checkpoint is reviewed.
