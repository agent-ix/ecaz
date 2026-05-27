# Artifact Manifest: Task 63 HNSW RaBitQ Suite Audit Handoff

- head SHA: pending commit
- task bucket: `reviews/task-63/022-hnsw-rabitq-suite-audit-handoff/`
- lane: HNSW RaBitQ benchmark handoff config audit
- fixture: `ec_real_50k`, `ec_real_100k`
- storage format: `turboquant`, `pq_fastscan`, `rabitq`
- rerank mode: unchanged
- timestamp: 2026-05-27T08:04:53-07:00
- isolated one-index-per-table surface: yes; inherited from the checked-in
  suite configs

## Artifacts

- `suite-audit-linux.log`: local static audit of `suite.json`; passed with 28
  steps.
- `suite-audit-m5-local.log`: local static audit of `suite-m5.json`; failed
  because this non-M5 host lacks `data/task31_m5_dbpedia_staged/` inputs.

## Commands

- `cargo run -q -p ecaz-cli -- bench suite audit --config benchmarks/task63-hnsw-rabitq-format/suite.json`
- `cargo run -q -p ecaz-cli -- bench suite audit --config benchmarks/task63-hnsw-rabitq-format/suite-m5.json`
