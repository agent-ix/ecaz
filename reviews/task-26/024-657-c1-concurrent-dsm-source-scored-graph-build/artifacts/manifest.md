# Artifact Manifest

## pg18_source_dsm_smoke.sql

- head SHA: `50290adca464f236eacd05c2ae1f6a6a2ae12639`
- packet/topic: `657-c1-concurrent-dsm-source-scored-graph-build`
- lane: PG18 source-scored concurrent DSM graph build smoke
- fixture: synthetic 2000-row, 16-dimensional `real[]` source vectors encoded to `ecvector`
- storage format: `ec_hnsw` scalar encoded-code index with `build_source_column = source`
- rerank mode: not applicable; build-only smoke
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file tmp/pg18_source_dsm_smoke.sql --log-output tmp/pg18_source_dsm_smoke.log`
- timestamp: `2026-04-25T22:32:57-07:00`
- isolated one-index-per-table or shared-table surface: isolated one-index table for the smoke fixture
- artifact files:
  - `pg18_source_dsm_smoke.sql`
  - `pg18_source_dsm_smoke.log`
- key result lines:
  - `CREATE INDEX`
  - `requested_workers = 2`
  - `workers_launched = 2`
  - `heap_tuples = 2000`
  - `index_tuples = 1998`
  - `graph_us = 305961`
  - `concurrent_dsm_graph_workers_launched = 2`
