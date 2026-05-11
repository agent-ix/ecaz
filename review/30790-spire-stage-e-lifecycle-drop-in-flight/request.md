# Review Request: SPIRE Stage E Lifecycle Drop In Flight

## Summary

This packet adds runtime evidence for the Stage E lifecycle row
`drop_remote_index_in_flight`.

Code checkpoint: `83983d9a867936e80663855b53b228522286d71e`

The fixture now supports a second lifecycle timing boundary: the coordinator
builds production candidate receive requests while the remote index exists,
then a pg-test helper runs remote `DROP INDEX` before production candidate
receive executes those requests. This distinguishes in-flight disappearance
from packet `30789`, which dropped the remote index before request
construction.

## Evidence

Artifacts are stored under `artifacts/`:

- `stage_e_lifecycle_drop_remote_index_in_flight.log`
- `stage_e_lifecycle_drop_remote_index_in_flight_strict.log`
- `stage_e_lifecycle_drop_remote_index_in_flight_degraded.log`
- `remote-ready-postgres.log`
- `coord-postgres.log`
- `manifest.md`

Strict mode key lines:

```text
injection=DROP INDEX ec_spire_stage_e_lifecycle_dropped_idx after request construction before receive
dropped_index_to_regclass_is_null=t
observed_candidate_receive_rows=2,remote_candidate_receive_failed,remote_index_unavailable,0
3,ready,none,1
observed_summary=spire_remote_fanout_executor_v1,2,2,1,1,remote_index_unavailable,1,0,none,compact_candidate_receive,remote_candidate_receive_failed
```

Degraded mode key lines:

```text
injection=DROP INDEX ec_spire_stage_e_lifecycle_dropped_idx after request construction before receive
dropped_index_to_regclass_is_null=t
observed_candidate_receive_rows=2,remote_candidate_receive_failed,remote_index_unavailable,0
3,ready,none,1
observed_summary=spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,remote_index_unavailable,remote_heap_resolution,degraded_ready
```

## Validation

- `bash -n scripts/run_spire_multicluster_stage_e_lifecycle_pg18.sh`
- `cargo fmt --check`
- `git diff --check -- src/lib.rs crates/ecaz-cli/src/commands/dev/spire_multicluster.rs crates/ecaz-cli/src/cli.rs scripts/run_spire_multicluster_stage_e_lifecycle_pg18.sh plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `cargo check --no-default-features --features pg18,pg_test`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli spire_multicluster -- --nocapture`
- `cargo run -p ecaz-cli -- dev spire-multicluster lifecycle-pg18 --case drop_remote_index_in_flight --artifact-dir review/30790-spire-stage-e-lifecycle-drop-in-flight/artifacts --run-id 30790`

## Review Focus

- Whether the new pg-test helper proves the intended timing boundary: request
  construction before injected remote DDL, then production candidate receive.
- Whether strict and degraded summaries match the Stage E lifecycle matrix for
  `remote_index_unavailable`.
- Whether the CLI/script extension keeps the lifecycle fixture composable for
  the remaining REINDEX and CREATE INDEX CONCURRENTLY rows.
