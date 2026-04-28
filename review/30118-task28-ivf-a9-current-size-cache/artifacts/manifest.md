# Artifact Manifest

## ivf_100k_n128_size_snapshot.sql

- head SHA: `3a446b19`
- packet/topic: `30118-task28-ivf-a9-current-size-cache`
- lane: Task 28 IVF A9 selected operating point storage/cache-state snapshot
- fixture: `task28_ivf_pqg100k_g8_n128`
- storage format: `pq_fastscan`
- PQ group size: `8`
- nlists: `128`
- rerank mode: `heap_f32`
- rerank width: `500`
- timestamp: `2026-04-28T13:00:10-07:00`
- surface: shared-table 100k ec_ivf benchmark surface created by earlier Task 28 packets
- cache state: warm local development run; no explicit OS or PostgreSQL buffer cache drop
- command:
  `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30118-task28-ivf-a9-current-size-cache/artifacts/ivf_100k_n128_size_snapshot.sql --raw --log-output review/30118-task28-ivf-a9-current-size-cache/artifacts/ivf_100k_n128_size_snapshot.log`
- key result:
  `task28_ivf_pqg100k_g8_n128_idx | ec_ivf | index_bytes 19791872 | 19 MB`
