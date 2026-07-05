# Task 71 / Packet 006 Artifact Manifest

- Head SHA: `59aa48890`
- Task bucket: `reviews/task-71/`
- Packet path: `reviews/task-71/006-single-tid-posting/`
- Slice: Single-TID IVF build posting staging
- Storage format: `pq_fastscan`
- Rerank mode: `heap_f32`
- Surface: local PG18 probe, isolated one-index-per-table prefix
- Timestamp: 2026-06-03 America/Los_Angeles

## Artifacts

### `install-after-single-tid-posting.log`

- Command:
  `./target/debug/ecaz --log-file reviews/task-71/006-single-tid-posting/artifacts/install-after-single-tid-posting.log dev install ecaz-pg-test --pg 18`
- Result: passed
- Key lines:
  - `Finished installing ecaz`
  - `backend artifact assertion passed`
  - `sha256=2b8c5b56a3f1cf0dbee74401e6009f0a5a340c7887f130ed1b209ad92a749ade`

### `probe-single-tid-posting-w8.log`

- Command:
  `./target/debug/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-71/006-single-tid-posting/artifacts/probe-single-tid-posting-w8.log dev test ivf-parallel-build-probe --drop-first --workers 8 --prefix task71_probe_w8_single_tid_posting`
- Result: passed
- Surface: isolated one-index-per-table probe
- Key lines:
  - `built task71_probe_w8_single_tid_posting_idx in 417.46ms`
  - `requested_workers=8 workers_launched=7 heap_tuples=10000 index_tuples=10000 heap_ingest_us=35622 train_model_us=263644 stage_build_plan_us=108117 stage_pq_train_us=15811 stage_centroids_us=186 stage_assign_us=29179 stage_postings_us=62898 stage_directory_us=4 flush_build_plan_us=2278`

### `probe-load-real10k-w8-single-tid-posting.log`

- Emitted by the probe command above.
- Result: passed
- Key lines:
  - source corpus/query SHA lines for Task 31 staged real10k inputs
  - `built task71_probe_w8_single_tid_posting_idx in 417.46ms`
  - expanded `ec_ivf build timing` row with `stage_postings_us=62898`
