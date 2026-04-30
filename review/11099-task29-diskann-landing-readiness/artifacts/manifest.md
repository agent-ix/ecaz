# Artifact Manifest

Head SHA: `b1cee686154114fc5e15665ad99f45f8e5a1feb7`

Packet: `review/11099-task29-diskann-landing-readiness`

Lane: Task 29 DiskANN initial tuning, local PG18 landing readiness.

Fixture: local PG18, real-10k 1536-d corpus for benchmark artifacts, focused
PG18 pgrx callback tests for correctness smoke.

Storage format: `ec_diskann` `pq_fastscan` tuple format with binary-sidecar
prefilter from Task 29a.

Rerank mode: heap-f32 exact rerank, existing reloption default
`rerank_budget=64`.

Table model: isolated one-index-per-table prefix for real-10k benchmark
artifacts.

Cache state: callback smoke is functional coverage. Benchmark artifacts copied
from packets `11096` and `11098` used warm local PG18 cache state as recorded in
their source manifests.

## Artifacts

### `pg18-diskann-callback-smoke.log`

Head SHA: `b1cee686154114fc5e15665ad99f45f8e5a1feb7`

Packet/topic: `review/11099-task29-diskann-landing-readiness`

Table model: pg_test-managed callback fixtures, not a benchmark corpus.

Command:

`script -q -e -c "cargo pgrx test pg18 test_ec_diskann_" review/11099-task29-diskann-landing-readiness/artifacts/pg18-diskann-callback-smoke.log`

Timestamp: 2026-04-30 local.

Key result lines:

- `running 19 tests`
- `test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 860 filtered out; finished in 53.45s`

Coverage represented by passing test names:

- `pg_test_ec_diskann_build_coalesces_duplicate_vectors`
- `pg_test_ec_diskann_sql_ordered_index_scan_executes`
- `pg_test_ec_diskann_sql_limit_can_exceed_reloption_top_k`
- `pg_test_ec_diskann_unique_insert_is_scan_reachable`
- `pg_test_ec_diskann_duplicate_after_append_binds_existing_node`
- `pg_test_ec_diskann_duplicate_insert_binds_first_overflow_tuple`
- `pg_test_ec_diskann_duplicate_bind_grows_second_overflow_tuple`
- `pg_test_ec_diskann_empty_index_bootstrap_insert_executes`
- `pg_test_ec_diskann_empty_index_remains_planner_gated`
- `pg_test_ec_diskann_session_list_size_override_changes_scan_width`
- `pg_test_ec_diskann_planner_prefers_seqscan_for_small_tables`
- `pg_test_ec_diskann_planner_chooses_index_scan_for_large_table`
- `pg_test_ec_diskann_vacuum_noop_stats_on_empty_index`
- `pg_test_ec_diskann_vacuum_promotes_duplicate_overflow_primary`
- `pg_test_ec_diskann_vacuum_unlinks_and_tombstones_dead_node`
- `pg_test_ec_diskann_vacuum_sets_medoid_refresh_flag`
- `pg_test_ec_diskann_vacuum_refills_broken_neighbor_slot`
- `pg_test_ec_diskann_full_backlink_rewrite_keeps_insert_reachable`
- `pg_test_ec_diskann_vacuum_replans_on_stale_repair_tuple`

### `load-task29a-sidecar-real10k.log`

Source: copied from `review/11096-task29a-diskann-binary-sidecar-prefilter`.

Original measured head:
`6491aeb60a6905ff546f117ce5d6d14d032059b4`

Original command: recorded in
`review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/manifest.md`.

Key result lines:

- copy time `4.27s`
- encode time `4.55s`
- index build time `492.13s`
- total load time `503.10s`

### `recall-task29a-sidecar-fresh-table.log`

Source: copied from `review/11096-task29a-diskann-binary-sidecar-prefilter`.

Original measured head:
`6491aeb60a6905ff546f117ce5d6d14d032059b4`

Original command: recorded in
`review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/manifest.md`.

Key result rows:

- L=64: recall@10 `0.9965`, NDCG `0.9997`, mean `68.58 ms`
- L=128: recall@10 `0.9960`, NDCG `0.9999`, mean `70.95 ms`
- L=200: recall@10 `0.9970`, NDCG `0.9999`, mean `70.23 ms`
- L=400: recall@10 `0.9970`, NDCG `0.9999`, mean `121.96 ms`
- L=800: recall@10 `0.9975`, NDCG `0.9999`, mean `279.63 ms`

### `recall-sidecar-auto-table.log`

Source: copied from `review/11096-task29a-diskann-binary-sidecar-prefilter`.

Original measured head:
`6491aeb60a6905ff546f117ce5d6d14d032059b4`

Original command: recorded in
`review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/manifest.md`.

Key result rows used for before/after latency comparison:

- L=64: recall@10 `0.9955`, NDCG `0.9997`, mean `52.87 ms`
- L=128: recall@10 `0.9960`, NDCG `0.9999`, mean `56.50 ms`
- L=200: recall@10 `0.9970`, NDCG `0.9999`, mean `67.65 ms`
- L=400: recall@10 `0.9970`, NDCG `0.9999`, mean `109.07 ms`
- L=800: recall@10 `0.9975`, NDCG `0.9999`, mean `247.34 ms`

### `latency-sidecar-auto-table.log`

Source: copied from `review/11096-task29a-diskann-binary-sidecar-prefilter`.

Original measured head:
`6491aeb60a6905ff546f117ce5d6d14d032059b4`

Original command: recorded in
`review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/manifest.md`.

Key result row:

- L=200: mean `68.0 ms`, p50 `66.6 ms`, p95 `71.7 ms`, p99 `73.5 ms`, HWM `70468 KiB`

### `storage-task29a-sidecar-fresh-cli.log`

Source: copied from `review/11096-task29a-diskann-binary-sidecar-prefilter`.

Original measured head:
`6491aeb60a6905ff546f117ce5d6d14d032059b4`

Original command: recorded in
`review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/manifest.md`.

Key result row:

- DiskANN index size `4.7 MiB`, bytes per row `494.0 B`

### `recall-sidecar-early-stop-table.log`

Source: copied from `review/11098-task29-diskann-early-stop-latency`.

Original measured head:
`27bb6af8a037b29918f13ca894cc1c1a466c834d`

Original command: recorded in
`review/11098-task29-diskann-early-stop-latency/artifacts/manifest.md`.

Key result rows:

- L=64: recall@10 `0.9955`, NDCG `0.9997`, mean `50.36 ms`
- L=128: recall@10 `0.9960`, NDCG `0.9999`, mean `48.80 ms`
- L=200: recall@10 `0.9970`, NDCG `0.9999`, mean `53.15 ms`
- L=400: recall@10 `0.9970`, NDCG `0.9999`, mean `58.89 ms`
- L=800: recall@10 `0.9975`, NDCG `0.9999`, mean `68.90 ms`

### `latency-sidecar-early-stop-table.log`

Source: copied from `review/11098-task29-diskann-early-stop-latency`.

Original measured head:
`27bb6af8a037b29918f13ca894cc1c1a466c834d`

Original command: recorded in
`review/11098-task29-diskann-early-stop-latency/artifacts/manifest.md`.

Key result rows:

- L=64: mean `48.5 ms`, p50 `47.8 ms`, p95 `54.1 ms`, p99 `57.0 ms`, HWM `65024 KiB`
- L=128: mean `54.1 ms`, p50 `50.3 ms`, p95 `76.3 ms`, p99 `88.7 ms`, HWM `64544 KiB`
- L=200: mean `58.5 ms`, p50 `55.9 ms`, p95 `75.0 ms`, p99 `90.1 ms`, HWM `64544 KiB`
- L=400: mean `61.7 ms`, p50 `61.2 ms`, p95 `74.6 ms`, p99 `82.9 ms`, HWM `65268 KiB`
- L=800: mean `67.7 ms`, p50 `66.7 ms`, p95 `76.9 ms`, p99 `80.0 ms`, HWM `66640 KiB`

### `recall-ec-hnsw-reference-table.log`

Source: copied from `review/11096-task29a-diskann-binary-sidecar-prefilter`.

Original measured head:
`6491aeb60a6905ff546f117ce5d6d14d032059b4`

Original command: recorded in
`review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/manifest.md`.

Key result row:

- ef=200: recall@10 `0.9700`, NDCG `0.9993`, mean `35.25 ms`

### `latency-ec-hnsw-reference-table.log`

Source: copied from `review/11096-task29a-diskann-binary-sidecar-prefilter`.

Original measured head:
`6491aeb60a6905ff546f117ce5d6d14d032059b4`

Original command: recorded in
`review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/manifest.md`.

Key result row:

- ef=200: mean `34.5 ms`, p50 `33.1 ms`, p95 `39.4 ms`, p99 `49.1 ms`, HWM `49028 KiB`

### `storage-ec-hnsw-reference-cli.log`

Source: copied from `review/11096-task29a-diskann-binary-sidecar-prefilter`.

Original measured head:
`6491aeb60a6905ff546f117ce5d6d14d032059b4`

Original command: recorded in
`review/11096-task29a-diskann-binary-sidecar-prefilter/artifacts/manifest.md`.

Key result row:

- HNSW index size `13.0 MiB`, bytes per row `1366.4 B`
