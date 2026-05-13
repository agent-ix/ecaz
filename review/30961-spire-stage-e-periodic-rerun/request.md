# Review Request: SPIRE Stage E Periodic Rerun

## Summary

Closes the Phase 12.7 row:

> Preserve and periodically rerun the full Stage E fault/lifecycle matrix
> against the current CustomScan path while this hardening proceeds.

This packet records a fresh local PG18 rerun against the current
`task-30-spire` branch:

- CustomScan read proof still plans through
  `Custom Scan (EcSpireDistributedScan)` and returns the remote row;
- 11/11 Stage E fault cases pass in strict and degraded mode; and
- 6/6 Stage E lifecycle cases pass in strict and degraded mode.

No executor behavior changed in this slice; the code checkpoint only marks the
tracker row closed with packet-local evidence.

## Files

- `plan/tasks/task30-phase12-spire-production-hardening.md`
- `review/30961-spire-stage-e-periodic-rerun/artifacts/...`

## Validation

Packet-local logs are in `artifacts/` and indexed by
`artifacts/manifest.md`.

- `bash scripts/run_spire_multicluster_customscan_read_pg18.sh --skip-install ...`
- `bash scripts/run_spire_multicluster_stage_e_network_partition_pg18.sh --skip-install ...`
- `bash scripts/run_spire_multicluster_stage_e_predispatch_fault_pg18.sh --case epoch_mismatch --skip-install ...`
- `bash scripts/run_spire_multicluster_stage_e_predispatch_fault_pg18.sh --case version_skew --skip-install ...`
- `bash scripts/run_spire_multicluster_stage_e_candidate_receive_fault_pg18.sh --case fingerprint_mismatch --skip-install ...`
- `bash scripts/run_spire_multicluster_stage_e_candidate_receive_fault_pg18.sh --case missing_or_reindexed_remote_index --skip-install ...`
- `bash scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh --case <transport-case> --skip-install ...`
- `bash scripts/run_spire_multicluster_stage_e_lifecycle_pg18.sh --case <lifecycle-case> --skip-install ...`
- `git diff --check 1939007c^ 1939007c`

The first two CustomScan read attempts were setup retries, not fixture evidence:
one used a relative `--run-dir`, and one ran inside the sandbox where PostgreSQL
could not bind Unix-domain sockets. The successful run used an absolute
`--run-dir` outside the sandbox, matching the Stage E fixture requirements.

## Reviewer Focus

- Confirm the packet-local logs cover the full 11 fault + 6 lifecycle matrix,
  not only the CustomScan smoke proof.
- Confirm the tracker closure is limited to a periodic rerun and does not claim
  remaining Phase 12 items such as typed tuple receive, placement contention,
  or INSERT 2PC cancellation are complete.
