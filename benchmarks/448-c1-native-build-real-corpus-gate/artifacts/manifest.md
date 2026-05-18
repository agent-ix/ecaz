# Artifact Manifest: 448-c1-native-build-real-corpus-gate

- head_sha: `81e86c011e118b4fc0169c3215675f773e5f35de`
- packet: `448-c1-native-build-real-corpus-gate`
- timestamp: `2026-04-19T13:09:15-07:00`

## Artifacts

### `real-50k-turboquant-gate.tsv`

- lane: `real 50k / turboquant`
- fixture: `tqhnsw_real_50k_corpus` + `tqhnsw_real_50k_queries_50`
- storage_format: `turboquant`
- rerank_mode: `source-backed build_source_column surface`
- surface: `four-config gate report`
- command:
  `./scripts/run_real_corpus_recall_scratch.sh --socket-dir /home/peter/.pgrx --port 28817 gate --prefix tqhnsw_real_50k --storage-format turboquant --queries-table tqhnsw_real_50k_queries_50`
- source_file:
  `/home/peter/dev/tqvector/tmp/real_corpus_runs/20260419T200355Z_gate_tqhnsw_real_50k_turboquant_tqhnsw_real_50k_queries_50.tsv`
- key_result_lines:
  - `8    40    0.886        t`
  - `8    128   0.93   0.89  t`
  - `8    200   0.93        t`
  - `16   200   0.964       t`

### `summary-helper-drift.log`

- lane: `real 50k / turboquant summary helper`
- fixture: `tqhnsw_real_50k_corpus` + `tqhnsw_real_50k_queries_50`
- storage_format: `requested turboquant`
- rerank_mode: `unexpected grouped heap-f32 path`
- surface: `failed targeted summary rerun`
- commands:
  - `./scripts/run_real_corpus_recall_scratch.sh --socket-dir /home/peter/.pgrx --port 28817 summary --index tqhnsw_real_50k_turboquant_m8_idx --m 8 --ef-search 128 --queries-table tqhnsw_real_50k_queries_50 --corpus-table tqhnsw_real_50k_corpus`
  - `./scripts/run_real_corpus_recall_scratch.sh --socket-dir /home/peter/.pgrx --port 28817 summary --index tqhnsw_real_50k_turboquant_m16_idx --m 16 --ef-search 200 --queries-table tqhnsw_real_50k_queries_50 --corpus-table tqhnsw_real_50k_corpus`
- key_result_lines:
  - `ERROR:  tqhnsw grouped heap-f32 rerank requires build_source_column, rerank_source_column, or TQVECTOR_PQ_FASTSCAN_RERANK_SOURCE_COLUMN...`
