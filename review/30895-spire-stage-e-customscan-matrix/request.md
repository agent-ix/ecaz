# 30895: SPIRE Stage E CustomScan Matrix Evidence

## Scope

This packet closes the Stage 11.5 CustomScan-pivot matrix migration tracker
item. It does not change executor behavior. It records current evidence that:

- the coordinator read path still plans and executes through
  `Custom Scan (EcSpireDistributedScan)` with tuple payloads after the cleanup
  packets; and
- the preserved Stage E executor fixtures still pass for all 11 fault cases and
  all 6 lifecycle cases after the AM materialization path was removed.

The Stage E fixtures intentionally keep asserting the executor state machine and
diagnostic SQL surfaces. Packets `30892` and `30894` removed the only
materialization-specific AM blocker path, so there was no remaining Stage E
fixture code to rewrite beyond rerunning the matrix against the current
CustomScan build.

## Validation

- `scripts/run_spire_multicluster_customscan_read_pg18.sh --artifact-dir review/30895-spire-stage-e-customscan-matrix/artifacts`
  - evidence: `artifacts/multicluster-customscan-read.log`
  - key lines: `Custom Scan (EcSpireDistributedScan)`,
    `read_row=10,remote alpha`, and
    `payload_probe=ready,2,{"id": 10, "title": "remote alpha"}`
- Stage E fault matrix, all strict/degraded cases:
  - `simulated_network_partition`
  - `epoch_mismatch`
  - `version_skew`
  - `fingerprint_mismatch`
  - `missing_or_reindexed_remote_index`
  - `connection_reset_mid_batch`
  - `local_cancel`
  - `local_statement_timeout`
  - `remote_backend_termination`
  - `remote_oom`
  - `remote_statement_timeout`
- Stage E lifecycle matrix, all strict/degraded cases:
  - `create_index_concurrently_missing_descriptor`
  - `create_index_concurrently_new_descriptor`
  - `drop_remote_index_before_fanout`
  - `drop_remote_index_in_flight`
  - `reindex_remote_index_before_fanout`
  - `reindex_remote_index_in_flight`
- `git diff --check`

## Review Focus

- Confirm the task tracker now accurately represents the CustomScan pivot:
  Stage 11.5 matrix migration is complete, while broader Stage E/Phase 11 items
  remain open.
- Confirm packet-local logs cover the full 11 fault + 6 lifecycle matrix and
  cite the CustomScan read proof, rather than relying on the superseded AM
  materialization path.
