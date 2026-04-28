# Artifact Manifest

## ivf_vacuum_replacement_reuse_smoke.sql

- head SHA: `d54e1f4037430e11d9fb0c18371634eb2b6dbcad`
- packet/topic: `30106-task28-ivf-vacuum-replacement-reuse-smoke`
- lane / fixture / storage format / rerank mode: IVF vacuum replacement / synthetic 5k 4D ecvector with repeated 2500-row population / heap_f32 postings / heap_f32 rerank
- command used: input SQL file for `ivf_vacuum_replacement_reuse_smoke.log`
- timestamp: `2026-04-28T17:31:00Z`
- isolation: isolated one-index-per-table surfaces for nlists 8, 32, and 64
- key cited lines: creates two copies of a 2500-row vector population, deletes one copy, runs `VACUUM (ANALYZE)`, inserts another copy, reports `pg_relation_size`

## ivf_vacuum_replacement_reuse_smoke.log

- head SHA: `d54e1f4037430e11d9fb0c18371634eb2b6dbcad`
- packet/topic: `30106-task28-ivf-vacuum-replacement-reuse-smoke`
- lane / fixture / storage format / rerank mode: IVF vacuum replacement / synthetic 5k 4D ecvector with repeated 2500-row population / heap_f32 postings / heap_f32 rerank
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30106-task28-ivf-vacuum-replacement-reuse-smoke/artifacts/ivf_vacuum_replacement_reuse_smoke.sql --raw --log-output review/30106-task28-ivf-vacuum-replacement-reuse-smoke/artifacts/ivf_vacuum_replacement_reuse_smoke.log`
- timestamp: `2026-04-28T17:31:00Z`
- isolation: isolated one-index-per-table surfaces for nlists 8, 32, and 64
- key cited lines:
  - after build: all three indexes `448 kB`
  - after delete vacuum: all three indexes `448 kB`
  - after refill: nlists=8 `448 kB`, nlists=32 `464 kB`, nlists=64 `536 kB`
  - VACUUM times: nlists=8 `17.023 ms`, nlists=32 `22.669 ms`, nlists=64 `65.239 ms`
