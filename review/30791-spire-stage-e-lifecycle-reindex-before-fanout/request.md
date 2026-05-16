# Review Request: SPIRE Stage E Lifecycle Reindex Before Fanout

## Summary

This packet adds runtime evidence for the Stage E lifecycle row
`reindex_remote_index_before_fanout`.

Code checkpoint: `8bb2fc83ba827492c6431afa9c5ba48de6d9721a`

The first fixture attempt exposed that the endpoint `profile_fingerprint` was
profile-only and did not change after `REINDEX INDEX CONCURRENTLY`. This slice
adds the index relation filenode to the endpoint fingerprint input, so stale
descriptors can detect physical index generation changes. The SQL surface still
returns the same 16-byte hex `profile_fingerprint` contract.

## Evidence

Artifacts are stored under `artifacts/`:

- `stage_e_lifecycle_reindex_remote_index_before_fanout.log`
- `stage_e_lifecycle_reindex_remote_index_before_fanout_strict.log`
- `stage_e_lifecycle_reindex_remote_index_before_fanout_degraded.log`
- `remote-ready-postgres.log`
- `coord-postgres.log`
- `manifest.md`

Strict mode key lines:

```text
dropped_remote_identity_before_drop=323e0c33462ad312
remote_reindexed_identity=32417233462db63b
observed_candidate_receive_rows=2,remote_candidate_receive_failed,endpoint_identity_mismatch,0
3,ready,none,1
observed_summary=spire_remote_fanout_executor_v1,2,2,1,1,endpoint_identity_mismatch,1,0,none,compact_candidate_receive,remote_candidate_receive_failed
```

Degraded mode key lines:

```text
dropped_remote_identity_before_drop=323e0c33462ad312
remote_reindexed_identity=32417233462db63b
observed_candidate_receive_rows=2,remote_candidate_receive_failed,endpoint_identity_mismatch,0
3,ready,none,1
observed_summary=spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,endpoint_identity_mismatch,remote_heap_resolution,degraded_ready
```

## Validation

- `bash -n scripts/run_spire_multicluster_stage_e_lifecycle_pg18.sh`
- `cargo fmt --check`
- `git diff --check -- src/am/ec_spire/root/remote_candidates.rs src/lib.rs crates/ecaz-cli/src/commands/dev/spire_multicluster.rs crates/ecaz-cli/src/cli.rs scripts/run_spire_multicluster_stage_e_lifecycle_pg18.sh plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `cargo check --no-default-features --features pg18,pg_test`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli spire_multicluster -- --nocapture`
- `cargo pgrx test pg18 test_ec_spire_remote_search_endpoint_identity`
- `cargo run -p ecaz-cli -- dev spire-multicluster lifecycle-pg18 --case reindex_remote_index_before_fanout --artifact-dir review/30791-spire-stage-e-lifecycle-reindex-before-fanout/artifacts --run-id 30791c`

## Review Focus

- Whether including `pg_relation_filenode(index_oid)` in the endpoint
  fingerprint is the right generation boundary for detecting remote REINDEX.
- Whether the lifecycle matrix now separates executor status
  `remote_candidate_receive_failed` from detection category
  `endpoint_identity_mismatch`.
- Whether the degraded pass correctly rewrites the remote reindexed endpoint to
  degraded mode before expecting a degraded skip.
