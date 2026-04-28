# Artifact Manifest

## ivf_vacuum_range_reuse_smoke.sql

- head SHA: `d54e1f4037430e11d9fb0c18371634eb2b6dbcad`
- packet/topic: `30105-task28-ivf-vacuum-range-reuse-smoke`
- lane / fixture / storage format / rerank mode: IVF vacuum churn / synthetic 5k 4D ecvector / heap_f32 postings / heap_f32 rerank
- command used: input SQL file for `ivf_vacuum_range_reuse_smoke.log`
- timestamp: `2026-04-28T17:26:00Z`
- isolation: isolated one-index-per-table surfaces for nlists 8, 32, and 64
- key cited lines: creates 5k rows per table, deletes 2500 rows, runs `VACUUM (ANALYZE)`, inserts 2500 replacement rows, reports `pg_relation_size`

## ivf_vacuum_range_reuse_smoke.log

- head SHA: `d54e1f4037430e11d9fb0c18371634eb2b6dbcad`
- packet/topic: `30105-task28-ivf-vacuum-range-reuse-smoke`
- lane / fixture / storage format / rerank mode: IVF vacuum churn / synthetic 5k 4D ecvector / heap_f32 postings / heap_f32 rerank
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30105-task28-ivf-vacuum-range-reuse-smoke/artifacts/ivf_vacuum_range_reuse_smoke.sql --raw --log-output review/30105-task28-ivf-vacuum-range-reuse-smoke/artifacts/ivf_vacuum_range_reuse_smoke.log`
- timestamp: `2026-04-28T17:26:00Z`
- isolation: isolated one-index-per-table surfaces for nlists 8, 32, and 64
- key cited lines:
  - after build: all three indexes `448 kB`
  - after delete vacuum: all three indexes `448 kB`
  - after refill: nlists=8 `464 kB`, nlists=32 `528 kB`, nlists=64 `576 kB`
  - VACUUM times: nlists=8 `17.094 ms`, nlists=32 `21.332 ms`, nlists=64 `27.812 ms`
