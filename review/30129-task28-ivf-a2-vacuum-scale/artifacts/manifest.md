# Artifacts Manifest

## ivf_vacuum_scale_1m_n8_32_64.log

- head SHA: `b2079361`
- packet/topic: `30129-task28-ivf-a2-vacuum-scale`
- lane: Task 28 IVF A2 streaming vacuum scale evidence
- fixture: synthetic 1M-row PG18 IVF vacuum scale harness
- storage format: `turboquant`
- rerank mode: `heap_f32`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 stress ivf-vacuum-scale --table-prefix task28_a2_vacuum_scale --rows 1000000 --nlists 8,32,64 --nprobe 8 --training-sample-rows 10000 --dimensions 4 --quantizer turboquant --sample-interval-ms 25 --log-output review/30129-task28-ivf-a2-vacuum-scale/artifacts/ivf_vacuum_scale_1m_n8_32_64.log`
- timestamp: 2026-04-28 16:05:24 PDT
- isolation: one table and one index per `nlists` value; shared local PG18 development database
- key result lines:
  - `8 | 1000000 | 500000 | vacuum_ms=2359 | idx_after_vacuum=89055232 | rss_peak_kb=364476 | hwm_peak_kb=426708 | memory_samples=88`
  - `32 | 1000000 | 500000 | vacuum_ms=2047 | idx_after_vacuum=89055232 | rss_peak_kb=370056 | hwm_peak_kb=432476 | memory_samples=77`
  - `64 | 1000000 | 500000 | vacuum_ms=2029 | idx_after_vacuum=89063424 | rss_peak_kb=370600 | hwm_peak_kb=433096 | memory_samples=75`
