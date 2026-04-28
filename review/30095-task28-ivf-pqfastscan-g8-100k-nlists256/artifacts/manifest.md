# Artifact Manifest

Head SHA: `e32f9b4fc70cc75df5d3e0aceb75abc0e081b492`

Packet: `review/30095-task28-ivf-pqfastscan-g8-100k-nlists256`

Timestamp: 2026-04-28 05:35 America/Los_Angeles

Fixture:

- Source prefix: `task28_ivf_pqg100k_g8`
- Derived prefix: `task28_ivf_pqg100k_g8_n256`
- Rows: 100000 corpus rows, 100 query rows
- Dimensions: 1536
- Isolated one-index-per-table surface: yes
- Storage format: `pq_fastscan`
- PQ group size: 8
- Rerank mode: `heap_f32`
- Rerank width: 500
- nlists: 256
- Runtime nprobe sweep: 64, 96, 128 for recall; 96, 128 for latency
- Cache state: warm local development run; no explicit cache drop
- Memory high-water mark: not captured

## build_g8_100k_n256_surface.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30095-task28-ivf-pqfastscan-g8-100k-nlists256/artifacts/build_g8_100k_n256_surface.sql --raw --log-output review/30095-task28-ivf-pqfastscan-g8-100k-nlists256/artifacts/build_g8_100k_n256_surface.log`

Key lines:

- `SELECT 100000` in `26609.538 ms`
- `CREATE INDEX` in `338805.986 ms`
- `task28_ivf_pqg100k_g8_n256_idx | 20 MB | {nlists=256,nprobe=256,training_sample_rows=2000,storage_format=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=500}`

## recall_g8_100k_n256_w500_p64_96_128.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n256 --profile ec_ivf --k 10 --queries-limit 100 --sweep 64,96,128 --force-index --log-output review/30095-task28-ivf-pqfastscan-g8-100k-nlists256/artifacts/recall_g8_100k_n256_w500_p64_96_128.log`

Key lines:

- Ground truth: `100 queries vs 100000 corpus rows (dim=1536)`, `20.71s`
- `64 | 0.9800 | 0.9988 | 205.89 ms`
- `96 | 0.9940 | 0.9998 | 263.65 ms`
- `128 | 0.9950 | 0.9998 | 332.51 ms`

## latency_g8_100k_n256_w500_p96_128.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg100k_g8_n256 --profile ec_ivf --k 10 --iterations 100 --sweep 96,128 --force-index --log-output review/30095-task28-ivf-pqfastscan-g8-100k-nlists256/artifacts/latency_g8_100k_n256_w500_p96_128.log`

Key lines:

- `96 | 100 | 272.3 ms | 20.7 ms | 235.5 ms | 270.1 ms | 305.7 ms | 342.2 ms | 354.9 ms`
- `128 | 100 | 333.5 ms | 22.3 ms | 292.9 ms | 330.0 ms | 370.6 ms | 416.6 ms | 427.1 ms`
