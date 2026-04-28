# Artifact Manifest

## ivf_vacuum_1m_n8_32_64.log

- head SHA: `3e5186f2`
- packet/topic: `30109-task28-ivf-a2-1m-vacuum-scale`
- lane: Task 28 IVF A2 VACUUM scale evidence
- fixture: synthetic 1,000,000-row isolated table per `nlists`, delete rows with `id > 500000`, then `VACUUM (ANALYZE)`
- storage format / quantizer: `turboquant`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 stress ivf-vacuum-scale --table-prefix task28_ivf_vacuum_1m --rows 1000000 --nlists 8,32,64 --nprobe 8 --training-sample-rows 10000 --dimensions 4 --quantizer turboquant --log-output review/30109-task28-ivf-a2-1m-vacuum-scale/artifacts/ivf_vacuum_1m_n8_32_64.log`
- timestamp: `2026-04-28T11:07:55-07:00`
- isolated/shared surface: isolated one-index-per-table surfaces
- key cited result lines:
  - `nlists=8`: `rows_before=1000000`, `rows_after=500000`, `delete_ms=434`, `vacuum_ms=2305`, `idx_before=89055232`, `idx_after_delete=89055232`, `idx_after_vacuum=89055232`, `rss_peak_kb=368328`, `hwm_peak_kb=430580`, `memory_samples=87`
  - `nlists=32`: `rows_before=1000000`, `rows_after=500000`, `delete_ms=295`, `vacuum_ms=2034`, `idx_before=89055232`, `idx_after_delete=89055232`, `idx_after_vacuum=89055232`, `rss_peak_kb=373188`, `hwm_peak_kb=435688`, `memory_samples=77`
  - `nlists=64`: `rows_before=1000000`, `rows_after=500000`, `delete_ms=322`, `vacuum_ms=2059`, `idx_before=89063424`, `idx_after_delete=89063424`, `idx_after_vacuum=89063424`, `rss_peak_kb=373724`, `hwm_peak_kb=436228`, `memory_samples=77`
