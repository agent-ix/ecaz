# Review Request: SPIRE Stage E Fingerprint Mismatch Fault

## Summary

This packet covers the Stage E `fingerprint_mismatch` runtime row.

The implementation extends the candidate-receive fixture family from packet
30781 to support a receive-time endpoint identity mismatch:

- `ecaz dev spire-multicluster fault-pg18 --case fingerprint_mismatch`
- documented candidate-receive fixture family cases at the top of
  `scripts/run_spire_multicluster_stage_e_candidate_receive_fault_pg18.sh`
- strict/degraded summary assertions for `endpoint_identity_mismatch`

## Evidence

Command:

```bash
cargo run -p ecaz-cli -- dev spire-multicluster fault-pg18 \
  --case fingerprint_mismatch \
  --artifact-dir review/30782-spire-stage-e-fingerprint-mismatch/artifacts \
  --run-id 30782 \
  --skip-install
```

Strict mode:

- mismatched-identity node: `remote_candidate_receive_failed,endpoint_identity_mismatch,0`
- ready node: `ready,none,1`
- summary: `spire_remote_fanout_executor_v1,2,2,1,1,endpoint_identity_mismatch,1,0,none,compact_candidate_receive,remote_candidate_receive_failed`

Degraded mode:

- mismatched-identity node: `remote_candidate_receive_failed,endpoint_identity_mismatch,0`
- ready node: `ready,none,1`
- summary: `spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,endpoint_identity_mismatch,remote_heap_resolution,degraded_ready`

The shared fixture was rerun for `missing_or_reindexed_remote_index` with
`--run-id 30781r --skip-install`; it still passed after the shared-script
changes.

Artifacts are listed in `artifacts/manifest.md`.

## Validation

- `bash -n scripts/run_spire_multicluster_stage_e_candidate_receive_fault_pg18.sh`
- `cargo fmt --check`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli spire_multicluster -- --nocapture`
- `git diff --check -- crates/ecaz-cli/src/cli.rs crates/ecaz-cli/src/commands/dev/spire_multicluster.rs scripts/run_spire_multicluster_stage_e_candidate_receive_fault_pg18.sh plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Review Focus

- Does the candidate-receive fixture model endpoint identity mismatch at the right layer?
- Are the strict/degraded assertions aligned with the Stage E fault matrix row?
- Is the fixture-family documentation enough to satisfy the 30779/30780 P3 feedback?
