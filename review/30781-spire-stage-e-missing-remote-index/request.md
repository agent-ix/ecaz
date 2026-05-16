# Review Request: SPIRE Stage E Missing Remote Index Fault

## Summary

This packet covers the Stage E `missing_or_reindexed_remote_index` runtime row.

The implementation adds:

- `ecaz dev spire-multicluster fault-pg18 --case missing_or_reindexed_remote_index`
- `scripts/run_spire_multicluster_stage_e_candidate_receive_fault_pg18.sh`
- pg-test-only candidate receive helpers that run the production libpq candidate receive path and summarize strict/degraded executor state
- scan handoff observability for degraded skip counters

## Evidence

Command:

```bash
cargo run -p ecaz-cli -- dev spire-multicluster fault-pg18 \
  --case missing_or_reindexed_remote_index \
  --artifact-dir review/30781-spire-stage-e-missing-remote-index/artifacts \
  --run-id 30781l \
  --skip-install
```

Strict mode:

- missing-index node: `remote_candidate_receive_failed,remote_index_unavailable,0`
- ready node: `ready,none,1`
- summary: `spire_remote_fanout_executor_v1,2,2,1,1,remote_index_unavailable,1,0,none,compact_candidate_receive,remote_candidate_receive_failed`

Degraded mode:

- missing-index node: `remote_candidate_receive_failed,remote_index_unavailable,0`
- ready node: `ready,none,1`
- summary: `spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,remote_index_unavailable,remote_heap_resolution,degraded_ready`

Artifacts are listed in `artifacts/manifest.md`.

## Validation

- `bash -n scripts/run_spire_multicluster_stage_e_candidate_receive_fault_pg18.sh`
- `cargo fmt --check`
- `cargo check --no-default-features --features pg18,pg_test`
- `cargo test -p ecaz-cli spire_multicluster -- --nocapture`
- `git diff --check -- crates/ecaz-cli/src/cli.rs crates/ecaz-cli/src/commands/dev/spire_multicluster.rs scripts/run_spire_multicluster_stage_e_candidate_receive_fault_pg18.sh src/am/ec_spire/root/remote_candidates.rs src/am/ec_spire/root/types.rs src/am/mod.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Review Focus

- Is the pg-test candidate receive helper narrow enough for fixture evidence without becoming production surface?
- Does the strict/degraded summary prove the intended missing/reindexed-index row?
- Is adding degraded skip counters to the scan handoff summary acceptable as production observability?
