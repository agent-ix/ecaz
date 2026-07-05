# Task 30 Packet 1066: AWS Operations Fault Restore

## Request

Please review the Phase 13e AWS operations fault-restore evidence captured on
the preserved packet 1062 Graviton cluster.

This packet intentionally preserves the first failed attempt at the packet root:
the harness still assumed a representative query row with `id = 0`, while the
real prepared query ids start at `100000`. Commit
`6f11d0c8a0434e775403ff14120240e8c448e74d` fixes the reusable fault harness to
select the first available query vector with `ORDER BY id LIMIT 1`.

The successful rerun lives under
`artifacts/rerun-after-query-vector-fix/`.

## Evidence Summary

- Reused the established AWS topology from packet 1062: `us-west-2`,
  `us-west-2a`, four `m7g.large` instances, one coordinator and three remotes.
  No provisioning, reinstall, data reload, or topology rebuild was performed.
- Degraded fault drill stopped remote node 2, returned remote heap candidates
  from the remaining two remotes, reported `status=degraded_ready`,
  `remote_heap_ready_dispatch_count=2`, `returned_candidate_count=10`, and
  `degraded_skipped_dispatch_count=1`.
- Strict fault drill stopped remote node 2 and failed closed as expected:
  `ec_spire remote write shape fingerprint failed to open connection for
  node_id 2`.
- Both fault paths restored remote node 2 through SSM/PostgreSQL restart,
  restarted the operator tunnel, and reached SQL readiness after one attempt.
- Final post-restore smoke returned to strict mode with
  `EcSpireDistributedScan`, `remote_fanout: 3`,
  `result_source=remote_heap_candidates`, `status=ready`,
  `remote_heap_ready_dispatch_count=3`, and zero timeout/cancel/degraded skips.
- All `us-west-2` pending/running/stopping instances were stopped and the
  final verification contains no active instance rows.

## Key Artifacts

- `artifacts/manifest.md`
- `artifacts/rerun-after-query-vector-fix/fault-degraded-session-summary.log`
- `artifacts/rerun-after-query-vector-fix/fault-degraded-assertion.log`
- `artifacts/rerun-after-query-vector-fix/fault-degraded.log`
- `artifacts/rerun-after-query-vector-fix/fault-strict-knn-strict.log`
- `artifacts/rerun-after-query-vector-fix/fault-strict.log`
- `artifacts/rerun-after-query-vector-fix/post-restore-smoke/smoke-customscan-read.log`
- `artifacts/rerun-after-query-vector-fix/post-restore-smoke/production-read-profile-smoke.log`
- `artifacts/rerun-after-query-vector-fix/post-restore-smoke/bench-spire-pipeline-smoke.log`
- `artifacts/rerun-after-query-vector-fix/aws-stop-verify-after-success.log`
