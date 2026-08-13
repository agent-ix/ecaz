# Packet 025 artifacts

- Task bucket: `reviews/task-167/025-review-fixes/`
- Code checkpoint containing the extension changes: `f0bcb06f8e50908a67568ce583d2e877103c3cc8`
- Fixture-only runner/comment checkpoint: `8fdfe828a`
- Run timestamp: `2026-08-13T10:04:55-07:00`
- Surface: isolated two-owner local PG18 multinode fixture; no shared-table
  benchmark surface; `--rows 100 --dim 4 --graph-degree 8`
- Installed extension: release profile, `pg18 pg_test`, exact SHA
  `f0bcb06f8e50908a67568ce583d2e877103c3cc8`
- Command:
  `/home/peter/.cargo-target/debug/ecaz dev distann-multicluster local-multinode-pg18 --pg 18 --pgbin /home/peter/.pgrx/18.3/pgrx-install/bin --nodes 2 --rows 100 --dim 4 --graph-degree 8 --skip-fault-drills --allow-debug-extension --remote-insert-probe --artifact-dir reviews/task-167/025-review-fixes/artifacts --log-file reviews/task-167/025-review-fixes/artifacts/exact-head-fixture-final.log`

## Durable result lines

- `artifacts/exact-head-fixture-final.log`: exact-head preflight passed;
  `role=saturated_target ... before_neighbors=Some(8) final_neighbors=Some(8) inserts_ok=true pass=true`;
  `role=frontier_retry_counter churn_retries=Some(0) steady_retries=Some(0) pass=true`;
  `forward_neighbors_selected=2 ... back_edge_check=true pass=true`;
  fixture summary `physical_concurrent_insert_query pass=true`.
- `artifacts/distann-multinode-summary.log`: fixture summary artifact.
- `artifacts/exact-head-fixture-saturated.log`: earlier exact-head diagnostic
  run showing the pre-gate `EC_RECORD_MISSING` repro and the missing random
  saturated-target source; retained to make the fix attribution auditable.

The packet intentionally excludes generated node PostgreSQL logs and other
operational exhaust. No benchmark result artifact is present because the
10k/50k/100k matrix has not yet been run.
