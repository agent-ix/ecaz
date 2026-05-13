# Artifact Manifest: SPIRE Local Readiness Bundle Attempt

- head SHA at run time: `1b815998f3b23b777e2647b10de3fd52fd92b3b0`
- packet/topic: `30978-spire-local-readiness-bundle`
- timestamp: `2026-05-13`
- evidence label: local production-readiness smoke attempt
- capacity profile: `docs/SPIRE_LOCAL_CAPACITY_TARGETS.md`
- status: partial evidence captured; final Phase 12 readiness bundle remains
  blocked by the findings in `request.md`.

## Passing Local Smoke Artifacts

### `customscan-read.log`

- command:
  `target/debug/ecaz dev spire-multicluster customscan-read-pg18 --artifact-dir review/30978-spire-local-readiness-bundle/artifacts/customscan-read --run-id phase12-final-customscan-read --coord-port 39180 --remote-port 39181 --smoke-log review/30978-spire-local-readiness-bundle/artifacts/customscan-read.log --log-file review/30978-spire-local-readiness-bundle/artifacts/customscan-read-cli.log`
- key result lines:
  - `plan=Limit -> Custom Scan (EcSpireDistributedScan)`
  - `read_row=10|remote alpha|{red,blue}|domain alpha|(7,left)`
  - `typed_payload_probe=ready,pg_binary_attr_v1,t,t`
  - `SPIRE multicluster CustomScan read passed`

### `insert-read-helper.log`

- command:
  `target/debug/ecaz dev spire-multicluster insert-read-after-customscan-pg18 --artifact-dir review/30978-spire-local-readiness-bundle/artifacts/insert-read-helper --run-id p12irh --coord-port 39189 --remote-port 39190 --insert-mode helper --skip-install --smoke-log review/30978-spire-local-readiness-bundle/artifacts/insert-read-helper.log --log-file review/30978-spire-local-readiness-bundle/artifacts/insert-read-helper-cli.log`
- key result lines:
  - `insert_result=2,remote_insert_prepared_pending_local_commit,await_local_commit,true,true`
  - `remote_row=303,remote inserted via coordinator`
  - `plan=Limit -> Custom Scan (EcSpireDistributedScan)`
  - `read_row=303,remote inserted via coordinator`
  - `SPIRE multicluster coordinator insert read-after-CustomScan passed`

### `transport-overlap.log`

- command:
  `target/debug/ecaz dev spire-multicluster transport-overlap-pg18 --artifact-dir review/30978-spire-local-readiness-bundle/artifacts/transport-overlap --run-id p12to --coord-port 39191 --remote-fast-port 39192 --remote-slow-port 39193 --skip-install --smoke-log review/30978-spire-local-readiness-bundle/artifacts/transport-overlap.log --log-file review/30978-spire-local-readiness-bundle/artifacts/transport-overlap-cli.log`
- key result lines:
  - `transport_overlap_row=2,ready,none,0,305,305,3`
  - `transport_overlap_row=3,ready,none,0,3,3,3`
  - `fast_completed_before_slow=true`
  - `SPIRE multicluster PG18 transport overlap passed`

### `fault-remote-statement-timeout.log`

- command:
  `target/debug/ecaz dev spire-multicluster fault-pg18 --case remote_statement_timeout --artifact-dir review/30978-spire-local-readiness-bundle/artifacts/fault-remote-statement-timeout --run-id p12rst --coord-port 39194 --remote-ready-port 39195 --skip-install --smoke-log review/30978-spire-local-readiness-bundle/artifacts/fault-remote-statement-timeout.log --log-file review/30978-spire-local-readiness-bundle/artifacts/fault-remote-statement-timeout-cli.log`
- key result lines:
  - strict summary reports `remote_transport_failed`,
    `first_transport_failure_category=remote_statement_timeout`
  - degraded summary reports `degraded_skipped_dispatch_count=1`,
    `first_degraded_skip_category=remote_statement_timeout`
  - `stage_e_fault_remote_statement_timeout_passed=true`

### `fault-local-cancel.log`

- command:
  `target/debug/ecaz dev spire-multicluster fault-pg18 --case local_cancel --artifact-dir review/30978-spire-local-readiness-bundle/artifacts/fault-local-cancel --run-id p12lc --coord-port 39196 --remote-ready-port 39197 --skip-install --smoke-log review/30978-spire-local-readiness-bundle/artifacts/fault-local-cancel.log --log-file review/30978-spire-local-readiness-bundle/artifacts/fault-local-cancel-cli.log`
- key result lines:
  - strict and degraded summaries report `cancelled_dispatch_count=2`
  - `first_cancellation_category=local_query_cancelled`
  - `stage_e_fault_local_cancel_passed=true`

## Bench / Metrics Artifacts

### `create-readiness-bench-fixture.sql`

- purpose: create a fresh PG18 local SPIRE fixture in database
  `spire_phase12_readiness`.
- final setup logs:
  - `drop-readiness-db-final.log`
  - `create-readiness-db-final.log`
  - `create-readiness-bench-fixture-final.log`
- key result lines:
  - `corpus_rows 600`
  - `query_rows 12`
  - `index_reloptions {nlists=1,nprobe=1,rerank_width=0,storage_format=rabitq}`

### `readiness-sql-metrics-final2.log`

- command:
  `target/debug/ecaz dev sql --host /home/peter/.pgrx --port 28818 --database spire_phase12_readiness --file review/30978-spire-local-readiness-bundle/artifacts/readiness-sql-metrics.sql --log-output review/30978-spire-local-readiness-bundle/artifacts/readiness-sql-metrics-final2.log`
- key result lines:
  - endpoint tuple transport: `pg_binary_attr_v1 ready {pg_binary_attr_v1} t`
  - latency/payload summary:
    `12 120 0.0000 8845 8.025 7.782 9.527 10.907`
  - pipeline rows include `candidates ready 600`, `heap_rerank ready 600`,
    and `remote_fanout not_applicable_local_scan`.
  - local store overlap row:
    `local_store_overlap 1 1 0 0 1 1 0 600 26772 1 0`

## Blocker Artifacts

### `insert-read.log` and `insert-read-rerun2.log`

- both trigger-mode live fixture attempts routed the row to the remote and read
  it through CustomScan, but exited nonzero because the coordinator table still
  contained id `303`.
- key line: `coordinator_row_count=1`.
- contrast: `cargo pgrx test pg18 test_ec_spire_enable_coordinator_insert_trigger_sql`
  passed in this session, so the mismatch appears specific to the live
  multicluster trigger fixture or transaction boundary.

### Bench CLI failure logs

- `spire-pipeline-readiness-bench-cli.log`: fresh local fixture bench recall
  failed while fetching exact truth with the v1 DML frontdoor guard.
- `spire-pipeline-query-metrics-only-typed-cli.log`: distributed tuple
  measurement query-metrics path failed with the same known guard recorded in
  packet `30975`.
- `spire-pipeline-local-bench-cli.log`: existing `tqvector_bench` corpus uses
  an older extension surface lacking
  `ec_spire_remote_search_endpoint_identity(oid)`.
