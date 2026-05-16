# Artifact Manifest: SPIRE Phase 12 Local Readiness Closeout

- source head before assembly: `9c0310362926178ed848f714e61b772a7da1351e`
- packet/topic: `30981-spire-phase12-local-readiness-closeout`
- timestamp: `2026-05-13`
- evidence label: local production-readiness smoke
- capacity profile: `docs/SPIRE_LOCAL_CAPACITY_TARGETS.md`
- evidence boundary: local PG18/pgrx fixtures only; no AWS/RDS product-scale
  claim is made by this packet.
- bundle shape: assembly manifest over packet-local raw logs already committed
  in packets `30978`, `30979`, and `30980`.

## Bundle Index

### Distributed CustomScan Read

- packet: `30978-spire-local-readiness-bundle`
- artifact:
  `review/30978-spire-local-readiness-bundle/artifacts/customscan-read.log`
- command:
  `target/debug/ecaz dev spire-multicluster customscan-read-pg18 --artifact-dir review/30978-spire-local-readiness-bundle/artifacts/customscan-read --run-id phase12-final-customscan-read --coord-port 39180 --remote-port 39181 --smoke-log review/30978-spire-local-readiness-bundle/artifacts/customscan-read.log --log-file review/30978-spire-local-readiness-bundle/artifacts/customscan-read-cli.log`
- key result lines:
  - `plan=Limit -> Custom Scan (EcSpireDistributedScan)`
  - `read_row=10|remote alpha|{red,blue}|domain alpha|(7,left)`
  - `typed_payload_probe=ready,pg_binary_attr_v1,t,t`
  - `SPIRE multicluster CustomScan read passed`

### Coordinator Write Then CustomScan Read

- packet: `30978-spire-local-readiness-bundle`
- artifact:
  `review/30978-spire-local-readiness-bundle/artifacts/insert-read-helper.log`
- command:
  `target/debug/ecaz dev spire-multicluster insert-read-after-customscan-pg18 --artifact-dir review/30978-spire-local-readiness-bundle/artifacts/insert-read-helper --run-id p12irh --coord-port 39189 --remote-port 39190 --insert-mode helper --skip-install --smoke-log review/30978-spire-local-readiness-bundle/artifacts/insert-read-helper.log --log-file review/30978-spire-local-readiness-bundle/artifacts/insert-read-helper-cli.log`
- key result lines:
  - `insert_result=2,remote_insert_prepared_pending_local_commit,await_local_commit,true,true`
  - `remote_row=303,remote inserted via coordinator`
  - `plan=Limit -> Custom Scan (EcSpireDistributedScan)`
  - `read_row=303,remote inserted via coordinator`
  - `SPIRE multicluster coordinator insert read-after-CustomScan passed`

### Trigger Write Then CustomScan Read

- packet: `30979-spire-trigger-live-fixture`
- artifact:
  `review/30979-spire-trigger-live-fixture/artifacts/insert-read-trigger-v2.log`
- command:
  `target/debug/ecaz dev spire-multicluster insert-read-after-customscan-pg18 --artifact-dir review/30979-spire-trigger-live-fixture/artifacts/insert-read-trigger-v2 --run-id p12trg2 --coord-port 39202 --remote-port 39203 --insert-mode trigger --skip-install --smoke-log review/30979-spire-trigger-live-fixture/artifacts/insert-read-trigger-v2.log --log-file review/30979-spire-trigger-live-fixture/artifacts/insert-read-trigger-v2-cli.log`
- key result lines:
  - `insert_result=trigger_insert_committed`
  - `coordinator_row_count=0`
  - `remote_row=303,remote inserted via coordinator`
  - `placement_row=2,3,2`
  - `plan=Limit -> Custom Scan (EcSpireDistributedScan)`
  - `read_row=303,remote inserted via coordinator`
  - `SPIRE multicluster coordinator insert read-after-CustomScan passed`

### Transport Overlap

- packet: `30978-spire-local-readiness-bundle`
- artifact:
  `review/30978-spire-local-readiness-bundle/artifacts/transport-overlap.log`
- command:
  `target/debug/ecaz dev spire-multicluster transport-overlap-pg18 --artifact-dir review/30978-spire-local-readiness-bundle/artifacts/transport-overlap --run-id p12to --coord-port 39191 --remote-fast-port 39192 --remote-slow-port 39193 --skip-install --smoke-log review/30978-spire-local-readiness-bundle/artifacts/transport-overlap.log --log-file review/30978-spire-local-readiness-bundle/artifacts/transport-overlap-cli.log`
- key result lines:
  - `transport_overlap_row=2,ready,none,0,305,305,3`
  - `transport_overlap_row=3,ready,none,0,3,3,3`
  - `fast_completed_before_slow=true`
  - `SPIRE multicluster PG18 transport overlap passed`

### Fault Checks

- packet: `30978-spire-local-readiness-bundle`
- artifacts:
  - `review/30978-spire-local-readiness-bundle/artifacts/fault-remote-statement-timeout.log`
  - `review/30978-spire-local-readiness-bundle/artifacts/fault-local-cancel.log`
- commands:
  - `target/debug/ecaz dev spire-multicluster fault-pg18 --case remote_statement_timeout --artifact-dir review/30978-spire-local-readiness-bundle/artifacts/fault-remote-statement-timeout --run-id p12rst --coord-port 39194 --remote-ready-port 39195 --skip-install --smoke-log review/30978-spire-local-readiness-bundle/artifacts/fault-remote-statement-timeout.log --log-file review/30978-spire-local-readiness-bundle/artifacts/fault-remote-statement-timeout-cli.log`
  - `target/debug/ecaz dev spire-multicluster fault-pg18 --case local_cancel --artifact-dir review/30978-spire-local-readiness-bundle/artifacts/fault-local-cancel --run-id p12lc --coord-port 39196 --remote-ready-port 39197 --skip-install --smoke-log review/30978-spire-local-readiness-bundle/artifacts/fault-local-cancel.log --log-file review/30978-spire-local-readiness-bundle/artifacts/fault-local-cancel-cli.log`
- key result lines:
  - `remote_transport_failed`
  - `first_transport_failure_category=remote_statement_timeout`
  - `degraded_skipped_dispatch_count=1`
  - `first_degraded_skip_category=remote_statement_timeout`
  - `stage_e_fault_remote_statement_timeout_passed=true`
  - `cancelled_dispatch_count=2`
  - `first_cancellation_category=local_query_cancelled`
  - `stage_e_fault_local_cancel_passed=true`

### Bench, Recall, Tuple Transport, And Local Store Counters

- packet: `30980-spire-dml-frontdoor-read-pass-through`
- artifact:
  `review/30980-spire-dml-frontdoor-read-pass-through/artifacts/spire-pipeline-readiness-bench-final.log`
- fixture: local PG18 database `spire_phase12_readiness`, prefix
  `phase12_ready`, index `phase12_ready_idx`, storage format `rabitq`,
  full retained frontier (`--rerank-width 0`), one-index-per-table local scan.
- command:
  `target/debug/ecaz bench spire-pipeline --host /home/peter/.pgrx --port 28818 --database spire_phase12_readiness --prefix phase12_ready --queries-limit 12 --sweep 1 --rerank-width 0 --include-query-metrics --include-recall --query-metric-k 10 --query-metric-projection-columns source --include-local-store-overlap --log-output review/30980-spire-dml-frontdoor-read-pass-through/artifacts/spire-pipeline-readiness-bench-final.log --log-file review/30980-spire-dml-frontdoor-read-pass-through/artifacts/spire-pipeline-readiness-bench-final-cli.log`
- key result lines:
  - endpoint tuple transport: `tuple_transport_status ready`,
    `pg_binary_attr_v1_ready true`
  - pipeline rows: `routing ready`, `placement ready`, `prefetch ready`,
    `candidates ready`, `heap_rerank ready`,
    `remote_fanout not_applicable_local_scan`
  - local store overlap: `candidate_sum 7200`, `object_bytes_sum 321264`,
    `read_batch_sum 12`
  - query metrics: `queries 12`, `latency_p50 8.313 ms`,
    `latency_p95 9.198 ms`, `latency_p99 9.330 ms`,
    `recall@k 1.0000`

### Recall Spot Check

- packet: `30980-spire-dml-frontdoor-read-pass-through`
- artifact:
  `review/30980-spire-dml-frontdoor-read-pass-through/artifacts/recall-q1-final.log`
- command:
  `target/debug/ecaz dev sql --host /home/peter/.pgrx --port 28818 --database spire_phase12_readiness --sql "SET enable_seqscan = off; SET enable_bitmapscan = off; SET enable_sort = off; SET ec_spire.nprobe = 1; SET ec_spire.rerank_width = 0; SELECT 'predicted' AS label, string_agg(id::text, ',' ORDER BY rank) FROM (SELECT id, row_number() OVER () AS rank FROM phase12_ready_corpus ORDER BY embedding <#> (SELECT source FROM phase12_ready_queries WHERE id = 1) LIMIT 10) s; SET enable_seqscan = on; SET enable_indexscan = off; SET enable_bitmapscan = off; SET enable_sort = on; SELECT 'exact' AS label, string_agg(id::text, ',' ORDER BY rank) FROM (SELECT id, row_number() OVER () AS rank FROM phase12_ready_corpus ORDER BY embedding <#> (SELECT source FROM phase12_ready_queries WHERE id = 1) LIMIT 10) s;" --log-output review/30980-spire-dml-frontdoor-read-pass-through/artifacts/recall-q1-final.log`
- key result lines:
  - `predicted 36,35,37,34,38,33,39,40,32,336`
  - `exact 32,33,34,35,36,37,38,39,40,336`

## Supporting Phase 12 Evidence

- placement write contention: packet `30969`, reviewed accepted.
- insert prepare local cancel: packet `30970`, reviewed accepted.
- async insert fanout decision: packet `30972`, reviewed accepted.
- batch/trigger prepare coverage: packet `30973`, reviewed accepted.
- schema drift, type round-trip, isolation, and negative DML fixtures are
  tracked as complete in `plan/tasks/task30-phase12-spire-production-hardening.md`
  with reviewer feedback through the Phase 12 packet sequence.
- libpq security and operations runbook: `docs/SPIRE_LIBPQ_RUNBOOK.md`,
  accepted in packet `30951`.
- local capacity targets: `docs/SPIRE_LOCAL_CAPACITY_TARGETS.md`, accepted in
  packet `30952`.
- local-vs-AWS evidence boundary: `docs/SPIRE_LOCAL_READINESS.md`, accepted in
  packet `30949`.

## Assembly Validation

- `rg -n -- "- \[ \]" plan/tasks/task30-phase12-spire-production-hardening.md`
  returned no unchecked tracker rows.
- `cargo fmt --check` passed after the 30980 reviewer follow-up code comment;
  it emitted the repository's existing stable-rustfmt warnings about unstable
  import options.
- `git diff --check` passed after the 30980 reviewer follow-up code comment.
