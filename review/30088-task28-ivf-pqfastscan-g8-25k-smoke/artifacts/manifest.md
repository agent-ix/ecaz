# Artifact Manifest

Head SHA: `6e44244`

Packet: `review/30088-task28-ivf-pqfastscan-g8-25k-smoke`

Timestamp: 2026-04-28 01:45 America/Los_Angeles

Fixture:

- Source prefix: `task28_ivf_postopt25k_n64w25`
- Derived prefix: `task28_ivf_pqg25k_g8`
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

## build_g8_25k_surface.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30088-task28-ivf-pqfastscan-g8-25k-smoke/artifacts/build_g8_25k_surface.sql --raw --log-output review/30088-task28-ivf-pqfastscan-g8-25k-smoke/artifacts/build_g8_25k_surface.log`

Key lines:

- `CREATE INDEX` in `47798.422 ms`
- `task28_ivf_pqg25k_g8_idx | 5176 kB | {nlists=64,nprobe=64,training_sample_rows=2000,storage_format=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=25}`

## recall_g8_25k_w25.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg25k_g8 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30088-task28-ivf-pqfastscan-g8-25k-smoke/artifacts/recall_g8_25k_w25.log`

Key lines:

- `32 | 0.7180 | 0.9827 | 83.96 ms`
- `48 | 0.7220 | 0.9834 | 113.90 ms`

## set_g8_25k_width250.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30088-task28-ivf-pqfastscan-g8-25k-smoke/artifacts/set_g8_25k_width250.sql --raw --log-output review/30088-task28-ivf-pqfastscan-g8-25k-smoke/artifacts/set_g8_25k_width250.log`

Key line:

- `task28_ivf_pqg25k_g8_idx | {nlists=64,nprobe=64,training_sample_rows=2000,storage_format=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=250}`

## recall_g8_25k_w250.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg25k_g8 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30088-task28-ivf-pqfastscan-g8-25k-smoke/artifacts/recall_g8_25k_w250.log`

Key lines:

- `32 | 0.9530 | 0.9977 | 91.99 ms`
- `48 | 0.9650 | 0.9989 | 119.30 ms`

## set_g8_25k_width1000.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30088-task28-ivf-pqfastscan-g8-25k-smoke/artifacts/set_g8_25k_width1000.sql --raw --log-output review/30088-task28-ivf-pqfastscan-g8-25k-smoke/artifacts/set_g8_25k_width1000.log`

Key line:

- `task28_ivf_pqg25k_g8_idx | {nlists=64,nprobe=64,training_sample_rows=2000,storage_format=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=1000}`

## recall_g8_25k_w1000.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg25k_g8 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30088-task28-ivf-pqfastscan-g8-25k-smoke/artifacts/recall_g8_25k_w1000.log`

Key lines:

- `32 | 0.9810 | 0.9988 | 126.46 ms`
- `48 | 0.9960 | 1.0000 | 150.85 ms`

## latency_g8_25k_w1000.log

Setup command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30088-task28-ivf-pqfastscan-g8-25k-smoke/artifacts/set_g8_25k_width1000.sql --raw --log-output review/30088-task28-ivf-pqfastscan-g8-25k-smoke/artifacts/set_g8_25k_width1000_for_latency.log`

Latency command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg25k_g8 --profile ec_ivf --k 10 --iterations 100 --sweep 32,48 --force-index --log-output review/30088-task28-ivf-pqfastscan-g8-25k-smoke/artifacts/latency_g8_25k_w1000.log`

Key lines:

- `32 | 100 | 124.9 ms | 5.65 ms | 117.9 ms | 124.4 ms | 130.2 ms | 133.6 ms | 170.3 ms`
- `48 | 100 | 154.6 ms | 6.65 ms | 144.7 ms | 153.2 ms | 164.1 ms | 175.8 ms | 194.5 ms`

## set_g8_25k_width25.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30088-task28-ivf-pqfastscan-g8-25k-smoke/artifacts/set_g8_25k_width25.sql --raw --log-output review/30088-task28-ivf-pqfastscan-g8-25k-smoke/artifacts/set_g8_25k_width25.log`

Restore-after-latency command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30088-task28-ivf-pqfastscan-g8-25k-smoke/artifacts/set_g8_25k_width25.sql --raw --log-output review/30088-task28-ivf-pqfastscan-g8-25k-smoke/artifacts/set_g8_25k_width25_after_latency.log`

Key line:

- `task28_ivf_pqg25k_g8_idx | {nlists=64,nprobe=64,training_sample_rows=2000,storage_format=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=25}`
