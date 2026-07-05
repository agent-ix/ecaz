# Review Request: SPIRE Local Readiness Bundle Attempt

- coder: coder1
- evidence commit: `068d1191b0cb74963b033c933e7cb4d32c8b4699`
- tracker rows: Phase 12.9 packet-local artifact capture and final local
  production-readiness bundle

## Scope

This packet is a local production-readiness bundle attempt, not a closeout. It
captures fresh packet-local evidence through the repo `target/debug/ecaz`
operator surface and identifies blockers that prevent honestly checking off the
remaining Phase 12.9 rows.

## Passing Evidence

- CustomScan distributed read passed with typed endpoint readiness:
  `typed_payload_probe=ready,pg_binary_attr_v1,t,t`.
- Coordinator-routed helper INSERT followed by CustomScan read passed:
  `remote_row=303,remote inserted via coordinator` and
  `read_row=303,remote inserted via coordinator`.
- Two-remote transport overlap passed:
  `fast_completed_before_slow=true`.
- Remote statement timeout fault passed in strict and degraded modes, including
  `remote_statement_timeout` strict failure and degraded-skip counters.
- Local cancel fault passed in strict and degraded modes, including
  `cancelled_dispatch_count=2` and
  `first_cancellation_category=local_query_cancelled`.
- Fresh local SQL metrics captured endpoint tuple transport readiness,
  p50/p95/p99 latency, payload bytes, pipeline route/candidate/heap rows,
  remote fanout status, and local-store object-byte/read-batch counters.

Artifact metadata and key lines are in `artifacts/manifest.md`.

## Blockers Found

1. Trigger-mode `insert-read-after-customscan-pg18` is not currently usable as a
   final bundle pass artifact. The live multicluster fixture routes the row to
   the remote and reads it back through CustomScan, but exits nonzero because
   `coordinator_row_count=1`. The focused PG18 test
   `cargo pgrx test pg18 test_ec_spire_enable_coordinator_insert_trigger_sql`
   passed in this session, so the mismatch needs investigation at the live
   fixture/transaction-boundary level.
2. `ecaz bench spire-pipeline` still cannot automate the current readiness
   measurements:
   - the distributed tuple-measurement DB hits the known v1 DML frontdoor guard
     on the tokio-postgres query-metrics path;
   - a fresh local readiness DB also hits that guard while fetching exact truth
     for recall;
   - the older `tqvector_bench` corpus has an older extension surface without
     `ec_spire_remote_search_endpoint_identity(oid)`.
3. The packet-local SQL recall sanity artifact is not acceptable as a readiness
   pass: even on the one-list/full-rerank local fixture it reports
   `recall_at_10 = 0.0000`. I did not mark the recall/artifact row complete.

## Validation Commands

- `target/debug/ecaz dev spire-multicluster customscan-read-pg18 ...`
- `target/debug/ecaz dev spire-multicluster insert-read-after-customscan-pg18 --insert-mode helper ...`
- `target/debug/ecaz dev spire-multicluster transport-overlap-pg18 ...`
- `target/debug/ecaz dev spire-multicluster fault-pg18 --case remote_statement_timeout ...`
- `target/debug/ecaz dev spire-multicluster fault-pg18 --case local_cancel ...`
- `target/debug/ecaz dev sql ... --file artifacts/create-readiness-bench-fixture.sql`
- `target/debug/ecaz dev sql ... --file artifacts/readiness-sql-metrics.sql`
- `cargo pgrx test pg18 test_ec_spire_enable_coordinator_insert_trigger_sql`

## Reviewer Focus

- Confirm the remaining Phase 12.9 rows should stay open.
- Confirm whether trigger-mode live fixture behavior is a product-path bug or a
  harness/transaction-boundary bug.
- Confirm whether the bench CLI needs to be widened before final readiness, or
  whether packet-local SQL artifacts are acceptable once recall is corrected.
