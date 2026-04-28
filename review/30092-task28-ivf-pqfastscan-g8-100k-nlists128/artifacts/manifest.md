# Artifact Manifest

Head SHA: `fb005c2`

Packet: `review/30092-task28-ivf-pqfastscan-g8-100k-nlists128`

Timestamp: 2026-04-28 04:05 America/Los_Angeles

Fixture:

- Source prefix: `task28_ivf_pqg100k_g8`
- Derived prefix: `task28_ivf_pqg100k_g8_n128`
- Rows: 100000 corpus rows, 100 query rows
- Dimensions: 1536
- Isolated one-index-per-table surface: yes
- Storage format: `pq_fastscan`
- PQ group size: 8
- Rerank mode: `heap_f32`
- Rerank width: 750
- nlists: 128
- Runtime nprobe sweep: 32, 48, 64, 96 for recall; 48, 64 for latency
- Cache state: warm local development run; no explicit cache drop
- Memory high-water mark: not captured

## build_g8_100k_n128_surface.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30092-task28-ivf-pqfastscan-g8-100k-nlists128/artifacts/build_g8_100k_n128_surface.sql --raw --log-output review/30092-task28-ivf-pqfastscan-g8-100k-nlists128/artifacts/build_g8_100k_n128_surface.log`

Key lines:

- `SELECT 100000` in `25802.013 ms`
- `CREATE INDEX` in `216414.112 ms`
- `task28_ivf_pqg100k_g8_n128_idx | 19 MB | {nlists=128,nprobe=128,training_sample_rows=2000,storage_format=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=750}`

## recall_g8_100k_n128_w750.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48,64,96 --force-index --log-output review/30092-task28-ivf-pqfastscan-g8-100k-nlists128/artifacts/recall_g8_100k_n128_w750.log`

Key lines:

- Ground truth: `100 queries vs 100000 corpus rows (dim=1536)`, `20.86s`
- `32 | 0.9710 | 0.9984 | 186.65 ms`
- `48 | 0.9920 | 0.9997 | 248.20 ms`
- `64 | 0.9940 | 0.9997 | 310.05 ms`
- `96 | 0.9980 | 0.9999 | 440.95 ms`

## latency_g8_100k_n128_w750.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 48,64 --force-index --log-output review/30092-task28-ivf-pqfastscan-g8-100k-nlists128/artifacts/latency_g8_100k_n128_w750.log`

Key lines:

- `48 | 100 | 253.3 ms | 15.1 ms | 221.2 ms | 251.5 ms | 279.8 ms | 290.1 ms | 292.6 ms`
- `64 | 100 | 314.3 ms | 19.9 ms | 282.5 ms | 311.9 ms | 352.8 ms | 372.4 ms | 389.6 ms`
