# Review Request: SPIRE Stage E Lifecycle Reindex In Flight

## Summary

This packet adds runtime evidence for the Stage E lifecycle row
`reindex_remote_index_in_flight`.

Code checkpoint: `83560b38db9a6b405692b6b1093c2d6c7e4d63a2`

The fixture builds production candidate receive requests against a live remote
endpoint identity, injects `REINDEX INDEX CONCURRENTLY` before receive, then
expects the live endpoint-generation fingerprint to reject the stale request
identity. During implementation, the degraded pass exposed that candidate
receive lacked the endpoint identity preflight already present in heap receive;
this slice adds that preflight so a rebuilt strict remote index cannot mask the
lifecycle boundary as a generic `remote_query_failed`.

## Evidence

Artifacts are stored under `artifacts/`:

- `stage_e_lifecycle_reindex_remote_index_in_flight.log`
- `stage_e_lifecycle_reindex_remote_index_in_flight_strict.log`
- `stage_e_lifecycle_reindex_remote_index_in_flight_degraded.log`
- `remote-ready-postgres.log`
- `coord-postgres.log`
- `manifest.md`

Strict mode key lines:

```text
injection=REINDEX INDEX CONCURRENTLY ec_spire_stage_e_lifecycle_dropped_idx after request construction before receive
dropped_remote_identity_before_drop=32636e33464a95d5
remote_reindexed_identity=4d87893355fdc4c5
observed_candidate_receive_rows=2,remote_candidate_receive_failed,endpoint_identity_mismatch,0
3,ready,none,1
observed_summary=spire_remote_fanout_executor_v1,2,2,1,1,endpoint_identity_mismatch,1,0,none,compact_candidate_receive,remote_candidate_receive_failed
```

Degraded mode key lines:

```text
injection=REINDEX INDEX CONCURRENTLY ec_spire_stage_e_lifecycle_dropped_idx after request construction before receive
dropped_remote_identity_before_drop=4d8aef335600a7ee
remote_reindexed_identity=4d79f13355f23821
observed_candidate_receive_rows=2,remote_candidate_receive_failed,endpoint_identity_mismatch,0
3,ready,none,1
observed_summary=spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,endpoint_identity_mismatch,remote_heap_resolution,degraded_ready
```

## Validation

- `bash -n scripts/run_spire_multicluster_stage_e_lifecycle_pg18.sh`
- `cargo fmt --check`
- `git diff --check -- src/am/ec_spire/root/remote_candidates.rs crates/ecaz-cli/src/commands/dev/spire_multicluster.rs crates/ecaz-cli/src/cli.rs scripts/run_spire_multicluster_stage_e_lifecycle_pg18.sh plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `cargo check --no-default-features --features pg18,pg_test`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli spire_multicluster -- --nocapture`
- `cargo run -p ecaz-cli -- dev spire-multicluster lifecycle-pg18 --case reindex_remote_index_in_flight --artifact-dir review/30792-spire-stage-e-lifecycle-reindex-in-flight/artifacts --run-id 30792b`

## Review Focus

- Whether candidate receive should always preflight endpoint identity before
  remote candidate SQL, matching heap receive.
- Whether the in-flight REINDEX fixture proves the request-build versus receive
  timing boundary.
- Whether strict/degraded summaries correctly classify the rebuilt endpoint as
  `endpoint_identity_mismatch`.
