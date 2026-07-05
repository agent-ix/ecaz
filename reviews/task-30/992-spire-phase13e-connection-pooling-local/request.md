# Review Request: Phase 13e Connection Pooling And Local Functionality Gates

Code commit: `402b92943a5a14149ee956b9cfbbdb2408c95fe5`

## Summary

This slice implements the evidence-triggered SPIRE production read connection pool and re-runs the local gates required before AWS resumes.

Key changes:

- Adds `ec_spire.remote_search_connection_pool_size` and a bounded backend-local pool for production candidate plus heap receive connections.
- Keys pooled connections by node descriptor generation, conninfo secret name, remote index regclass, remote index identity, TLS mode, user/db, statement timeout class, and conninfo fingerprint.
- Reuses only successful candidate+heap sessions; failures, endpoint identity mismatch, schema drift, disconnects, and pool eviction drop the connection.
- Updates production contracts/operator diagnostics from `per_query`/`no_pooling_v1` to `per_backend_reusable_idle_session`/`bounded_per_backend_v1`.
- Adds a final `HARNESS PASSED` marker to the local AWS-shape harness and makes AWS install wait for SSM readiness plus support coordinator-only conninfo refresh.

## Evidence

Primary local AWS-shape harness:

- `artifacts/phase13e-aws-harness-local.log`
- `artifacts/smoke-customscan-read.log`
- `artifacts/production-read-profile-smoke.log`
- `artifacts/bench-spire-pipeline-smoke.log`

Important lines:

- `HARNESS PASSED`
- `published_static_remote_placements`
- `Custom Scan (EcSpireDistributedScan)` with `remote_fanout: 3`
- `result_source remote_heap_candidates`
- `status ready`
- single production profile opens 3 sockets for the three remotes
- follow-up benchmark profiles show `socket_open_sum 0` with ready remote heap candidates, proving pooled reuse after warmup

Full local gate suite:

- `artifacts/local-gates-after-pooling/phase13e-local-gates-summary.tsv`

All 22 gates passed, including:

- static remote placement
- multicluster CustomScan read
- helper and trigger coordinator insert/readback
- transport overlap
- Stage E pre-dispatch, candidate, network partition, transport, and lifecycle faults

## Commands

```bash
cargo check --no-default-features --features pg18
bash scripts/run_spire_phase13e_aws_harness_local_pg18.sh --artifact-dir reviews/task-30/992-spire-phase13e-connection-pooling-local/artifacts
bash scripts/run_spire_phase13e_local_gates_pg18.sh --suite all --artifact-dir reviews/task-30/992-spire-phase13e-connection-pooling-local/artifacts/local-gates-after-pooling
```

## Notes

AWS testing did not run in this slice. The local functionality gate is now green; AWS correctness and representative latency/recall evidence remain pending and should stay on the established Graviton lane only.
