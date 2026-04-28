# Artifact Manifest

## ivf_vacuum_scale_smoke.log

- head SHA: `7bb54e7e`
- packet/topic: `30108-task28-ivf-vacuum-scale-harness`
- lane: Task 28 IVF VACUUM scale harness smoke
- fixture: synthetic 2,000-row isolated table per `nlists`
- storage format / quantizer: `turboquant`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 stress ivf-vacuum-scale --table-prefix task28_ivf_vacuum_scale_smoke --rows 2000 --nlists 8,32 --nprobe 8 --training-sample-rows 500 --dimensions 4 --quantizer turboquant --log-output review/30108-task28-ivf-vacuum-scale-harness/artifacts/ivf_vacuum_scale_smoke.log`
- timestamp: `2026-04-28T11:05:45-07:00`
- isolated/shared surface: isolated one-index-per-table surfaces
- key cited result lines:
  - `nlists=8`: `rows_before=2000`, `rows_after=1000`, `delete_ms=3`, `vacuum_ms=11`, `idx_before=188416`, `idx_after_delete=188416`, `idx_after_vacuum=188416`, `rss_peak_kb=33960`, `hwm_peak_kb=33960`, `memory_samples=1`
  - `nlists=32`: `rows_before=2000`, `rows_after=1000`, `delete_ms=2`, `vacuum_ms=15`, `idx_before=196608`, `idx_after_delete=196608`, `idx_after_vacuum=196608`, `rss_peak_kb=36520`, `hwm_peak_kb=36520`, `memory_samples=1`
