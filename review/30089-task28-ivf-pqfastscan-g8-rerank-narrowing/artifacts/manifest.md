# Artifact Manifest

Head SHA: `8a3458a`

Packet: `review/30089-task28-ivf-pqfastscan-g8-rerank-narrowing`

Timestamp: 2026-04-28 02:10 America/Los_Angeles

Fixture:

- Prefix: `task28_ivf_pqg25k_g8`
- Rows: 25000 corpus rows, 100 query rows
- Dimensions: 1536
- Isolated one-index-per-table surface: yes
- Storage format: `pq_fastscan`
- PQ group size: 8
- Rerank mode: `heap_f32`
- nlists: 64
- Runtime nprobe sweep: 32, 48
- Cache state: warm local development run; no explicit cache drop
- Memory high-water mark: not captured

## set_g8_25k_width500.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30089-task28-ivf-pqfastscan-g8-rerank-narrowing/artifacts/set_g8_25k_width500.sql --raw --log-output review/30089-task28-ivf-pqfastscan-g8-rerank-narrowing/artifacts/set_g8_25k_width500.log`

Key line:

- `task28_ivf_pqg25k_g8_idx | {nlists=64,nprobe=64,training_sample_rows=2000,storage_format=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=500}`

## recall_g8_25k_w500.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg25k_g8 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30089-task28-ivf-pqfastscan-g8-rerank-narrowing/artifacts/recall_g8_25k_w500.log`

Key lines:

- `32 | 0.9700 | 0.9985 | 102.55 ms`
- `48 | 0.9840 | 0.9998 | 130.12 ms`

## set_g8_25k_width750.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30089-task28-ivf-pqfastscan-g8-rerank-narrowing/artifacts/set_g8_25k_width750.sql --raw --log-output review/30089-task28-ivf-pqfastscan-g8-rerank-narrowing/artifacts/set_g8_25k_width750.log`

Key line:

- `task28_ivf_pqg25k_g8_idx | {nlists=64,nprobe=64,training_sample_rows=2000,storage_format=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=750}`

## recall_g8_25k_w750.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg25k_g8 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30089-task28-ivf-pqfastscan-g8-rerank-narrowing/artifacts/recall_g8_25k_w750.log`

Key lines:

- `32 | 0.9790 | 0.9987 | 115.97 ms`
- `48 | 0.9940 | 1.0000 | 140.85 ms`

## latency_g8_25k_w750.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg25k_g8 --profile ec_ivf --k 10 --iterations 100 --sweep 32,48 --force-index --log-output review/30089-task28-ivf-pqfastscan-g8-rerank-narrowing/artifacts/latency_g8_25k_w750.log`

Key lines:

- `32 | 100 | 115.6 ms | 7.05 ms | 104.2 ms | 114.1 ms | 127.9 ms | 133.1 ms | 159.8 ms`
- `48 | 100 | 141.2 ms | 5.38 ms | 133.7 ms | 140.0 ms | 146.3 ms | 151.3 ms | 183.8 ms`

## set_g8_25k_width25.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30089-task28-ivf-pqfastscan-g8-rerank-narrowing/artifacts/set_g8_25k_width25.sql --raw --log-output review/30089-task28-ivf-pqfastscan-g8-rerank-narrowing/artifacts/set_g8_25k_width25.log`

Key line:

- `task28_ivf_pqg25k_g8_idx | {nlists=64,nprobe=64,training_sample_rows=2000,storage_format=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=25}`
