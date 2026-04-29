# Artifact Manifest

## explain_scan_volume_990k_nprobe32_40_48_after_rerank_fix.log

- head SHA: `4426f1ff`
- packet/topic: `30135-task28-ivf-990k-scan-volume`
- lane: Task 28 IVF 990k scan-volume EXPLAIN probe
- fixture: existing DBPedia 990k IVF surface `task28_ivf_pqg990k_g8_n128`
- storage format: `pq_fastscan`, `pq_group_size=8`
- rerank mode: `heap_f32`, `rerank_width=500`
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30135-task28-ivf-990k-scan-volume/artifacts/explain_scan_volume_990k_nprobe32_40_48.sql --raw --log-output review/30135-task28-ivf-990k-scan-volume/artifacts/explain_scan_volume_990k_nprobe32_40_48_after_rerank_fix.log`
- timestamp: 2026-04-28T19:58:00-07:00
- surface: isolated one-index-per-table surface from packet 30130
- cache state: warm local PG18; no OS or Postgres cache drop
- key result lines:
  - nprobe 32: `Execution Time`: `788.092`, `Posting Pages Read`: `5795`, `Postings Visited`: `253879`, `Postings Scored`: `3228`, `Postings Pruned By Bound`: `250651`, `Candidates Inserted`: `3228`, `Rerank Rows`: `500`, `Shared Read Blocks`: `0`
  - nprobe 40: `Execution Time`: `893.385`, `Posting Pages Read`: `7212`, `Postings Visited`: `315958`, `Postings Scored`: `3232`, `Postings Pruned By Bound`: `312726`, `Candidates Inserted`: `3232`, `Rerank Rows`: `500`, `Shared Read Blocks`: `0`
  - nprobe 48: `Execution Time`: `1018.736`, `Posting Pages Read`: `8511`, `Postings Visited`: `372944`, `Postings Scored`: `3235`, `Postings Pruned By Bound`: `369709`, `Candidates Inserted`: `3235`, `Rerank Rows`: `500`, `Shared Read Blocks`: `0`

## explain_scan_volume_990k_nprobe32_40_48.log

- head SHA: pre-fix local run before commit `4426f1ff`
- packet/topic: `30135-task28-ivf-990k-scan-volume`
- lane: exploratory pre-fix scan-volume EXPLAIN probe
- fixture: existing DBPedia 990k IVF surface `task28_ivf_pqg990k_g8_n128`
- command: same SQL probe, logged to `explain_scan_volume_990k_nprobe32_40_48.log`
- timestamp: 2026-04-28T19:39:00-07:00
- surface: isolated one-index-per-table surface from packet 30130
- key result lines:
  - `Rerank Rows`: `0` at nprobe 32/40/48, which exposed the missing rerank counter increment fixed by commit `4426f1ff`

## admin_snapshot_990k.log

- head SHA: pre-fix local run before commit `4426f1ff`
- packet/topic: `30135-task28-ivf-990k-scan-volume`
- lane: exploratory admin snapshot attempt
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --sql "SELECT storage_format, rerank, relation_rerank_width, session_rerank_width, effective_rerank_width, effective_rerank_width_source FROM ec_ivf_index_admin_snapshot('task28_ivf_pqg990k_g8_n128_idx'::regclass::oid)" --raw --log-output review/30135-task28-ivf-990k-scan-volume/artifacts/admin_snapshot_990k.log`
- timestamp: 2026-04-28T19:44:00-07:00
- result: failed because this loaded database did not have the current SQL function installed; this artifact is not used for the measurement claim

## explain_scan_volume_990k_nprobe32_40_48.sql

- head SHA: `4426f1ff`
- packet/topic: `30135-task28-ivf-990k-scan-volume`
- lane: SQL input for the 990k scan-volume EXPLAIN probe
