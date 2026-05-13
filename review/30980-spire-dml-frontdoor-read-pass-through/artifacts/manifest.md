# Artifact Manifest: SPIRE Read Pass-Through and Local Recall Fix

- head SHA at run time: `f50482304336af6f2a01f47b96d72e778f60e847`
- packet/topic: `30980-spire-dml-frontdoor-read-pass-through`
- timestamp: `2026-05-13`
- lane: Phase 12.9 final local readiness blocker reconciliation
- fixture: local PG18 database `spire_phase12_readiness`, prefix
  `phase12_ready`, index `phase12_ready_idx`
- storage format: `rabitq`
- rerank mode: `ec_spire.rerank_width = 0` full retained frontier
- surface: isolated local one-index-per-table readiness fixture from packet
  `30978`; local scan path, no remote fanout

## `recall-q1-final.log`

- command:
  `target/debug/ecaz dev sql --host /home/peter/.pgrx --port 28818 --database spire_phase12_readiness --sql "SET enable_seqscan = off; SET enable_bitmapscan = off; SET enable_sort = off; SET ec_spire.nprobe = 1; SET ec_spire.rerank_width = 0; SELECT 'predicted' AS label, string_agg(id::text, ',' ORDER BY rank) FROM (SELECT id, row_number() OVER () AS rank FROM phase12_ready_corpus ORDER BY embedding <#> (SELECT source FROM phase12_ready_queries WHERE id = 1) LIMIT 10) s; SET enable_seqscan = on; SET enable_indexscan = off; SET enable_bitmapscan = off; SET enable_sort = on; SELECT 'exact' AS label, string_agg(id::text, ',' ORDER BY rank) FROM (SELECT id, row_number() OVER () AS rank FROM phase12_ready_corpus ORDER BY embedding <#> (SELECT source FROM phase12_ready_queries WHERE id = 1) LIMIT 10) s;" --log-output review/30980-spire-dml-frontdoor-read-pass-through/artifacts/recall-q1-final.log`
- key result lines:
  - `predicted 36,35,37,34,38,33,39,40,32,336`
  - `exact 32,33,34,35,36,37,38,39,40,336`
  - The top-10 sets match for the sampled query; ordering differs only among
    near-tie rows.

## `spire-pipeline-readiness-bench-final.log`

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

## Validation Commands

- `cargo test dml_frontdoor --lib`
  - result: 28 passed, including PG18 DML-frontdoor cases.
- `cargo test remote_heap_exact_score_uses_orderby_negative_inner_product --lib`
  - result: 1 passed.
- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`
  - result: installed updated PG18 extension; emitted pre-existing unused import
    warnings.
- `cargo fmt --check`
  - result: passed; emitted existing stable-rustfmt warnings about unstable
    import options.
- `git diff --check`
  - result: passed.
