# Task 71 / Packet 005 Artifact Manifest

- Head SHA: `3e2fe08fb`
- Task bucket: `reviews/task-71/`
- Packet path: `reviews/task-71/005-stage-subphase/`
- Slice: IVF stage subphase timing
- Storage format: `pq_fastscan`
- Rerank mode: `heap_f32`
- Surface: local PG18 probe, isolated one-index-per-table prefix
- Timestamp: 2026-06-03 America/Los_Angeles

## Artifacts

### `install-after-stage-subphase-timing.log`

- Command:
  `./target/debug/ecaz --log-file reviews/task-71/005-stage-subphase/artifacts/install-after-stage-subphase-timing.log dev install ecaz-pg-test --pg 18`
- Result: passed
- Key lines:
  - `Finished installing ecaz`
  - `backend artifact assertion passed`
  - `sha256=d76af3edbd7a31676e703fba8009a51b6676a95bd4f60d68cd9aa37d7f819a68`

### `probe-stage-subphase-w8.log`

- Command:
  `./target/debug/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-71/005-stage-subphase/artifacts/probe-stage-subphase-w8.log dev test ivf-parallel-build-probe --drop-first --workers 8 --prefix task71_probe_w8_stage_subphase`
- Result: passed
- Surface: isolated one-index-per-table probe
- Key lines:
  - `built task71_probe_w8_stage_subphase_idx in 463.36ms`
  - `requested_workers=8 workers_launched=7 heap_tuples=10000 index_tuples=10000 heap_ingest_us=35347 train_model_us=276084 stage_build_plan_us=144585 stage_pq_train_us=18071 stage_centroids_us=314 stage_assign_us=33218 stage_postings_us=92930 stage_directory_us=3 flush_build_plan_us=3477`

### `probe-load-real10k-w8-stage-subphase.log`

- Emitted by the probe command above.
- Result: passed
- Key lines:
  - source corpus/query SHA lines for Task 31 staged real10k inputs
  - `built task71_probe_w8_stage_subphase_idx in 463.36ms`
  - expanded `ec_ivf build timing` row with stage subphase fields
