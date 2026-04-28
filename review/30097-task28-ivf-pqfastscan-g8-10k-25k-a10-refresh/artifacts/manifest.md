# Artifact Manifest

Head SHA: `0653ca9ea393335afb272a28a54f59e9986fb3f3`

Packet: `review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh`

Timestamp: 2026-04-28 06:25 America/Los_Angeles

Fixture:

- Local PG18 database `postgres`.
- Existing isolated one-index-per-table surfaces.
- 10k rows / 100 queries for `task28_ivf_qcmp10k_turboquant` and
  `task28_ivf_pqg10k_g8`.
- 25k rows / 100 queries for `task28_ivf_postopt25k_n64w25` and
  `task28_ivf_pqg25k_g8`.
- Dimensions: 1536.
- Runtime nprobe sweep: 32, 48 for recall; 48 for latency.
- Cache state: warm local development run; no explicit cache drop.
- Memory high-water mark: not captured.

## inspect_surfaces.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/inspect_surfaces.sql --raw --log-output review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/inspect_surfaces.log`

Key lines:

- `task28_ivf_pqg10k_g8_idx | 2448 kB | ... storage_format=pq_fastscan,pq_group_size=8 ... rerank_width=25`
- `task28_ivf_pqg25k_g8_idx | 5176 kB | ... storage_format=pq_fastscan,pq_group_size=8 ... rerank_width=25`
- `task28_ivf_qcmp10k_turboquant_idx | 9416 kB | ... storage_format=turboquant ... rerank_width=25`
- `task28_ivf_postopt25k_n64w25_idx | 22 MB | ... storage_format=turboquant ... rerank_width=25`

## set_pqg8_width750.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/set_pqg8_width750.sql --raw --log-output review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/set_pqg8_width750.log`

Key lines:

- `task28_ivf_pqg10k_g8_idx | ... rerank_width=750`
- `task28_ivf_pqg25k_g8_idx | ... rerank_width=750`

## Current Width Recall Logs

Commands:

- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg10k_g8 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/recall10_pqg8_10k_w750.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp10k_turboquant --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/recall10_turbo_10k.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg25k_g8 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/recall10_pqg8_25k_w750.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_postopt25k_n64w25 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/recall10_turbo_25k.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg10k_g8 --profile ec_ivf --k 100 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/recall100_pqg8_10k_w750.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp10k_turboquant --profile ec_ivf --k 100 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/recall100_turbo_10k.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg25k_g8 --profile ec_ivf --k 100 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/recall100_pqg8_25k_w750.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_postopt25k_n64w25 --profile ec_ivf --k 100 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/recall100_turbo_25k_w25.log`

Key p48 lines:

- 10k PQ g8 width750 recall@10: `48 | 0.9910 | 0.9997 | 86.80 ms`
- 10k TurboQuant width25 recall@10: `48 | 1.0000 | 1.0000 | 85.70 ms`
- 25k PQ g8 width750 recall@10: `48 | 0.9940 | 1.0000 | 152.36 ms`
- 25k TurboQuant width25 recall@10: `48 | 0.9990 | 1.0000 | 204.09 ms`
- 10k PQ g8 width750 recall@100: `48 | 0.9360 | 0.9967 | 124.21 ms`
- 10k TurboQuant width25 recall@100: `48 | 0.2500 | 0.4515 | 92.58 ms`
- 25k PQ g8 width750 recall@100: `48 | 0.9256 | 0.9966 | 187.78 ms`
- 25k TurboQuant width25 recall@100: `48 | 0.2500 | 0.4430 | 209.50 ms`

## set_turbo_width750.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/set_turbo_width750.sql --raw --log-output review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/set_turbo_width750.log`

Key lines:

- `task28_ivf_qcmp10k_turboquant_idx | ... rerank_width=750`
- `task28_ivf_postopt25k_n64w25_idx | ... rerank_width=750`

## Matched Width750 Recall Logs

Commands:

- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp10k_turboquant --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/recall10_turbo_10k_w750.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_postopt25k_n64w25 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/recall10_turbo_25k_w750.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp10k_turboquant --profile ec_ivf --k 100 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/recall100_turbo_10k_w750.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_postopt25k_n64w25 --profile ec_ivf --k 100 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/recall100_turbo_25k_w750.log`

Key p48 lines:

- 10k TurboQuant width750 recall@10: `48 | 1.0000 | 1.0000 | 120.89 ms`
- 25k TurboQuant width750 recall@10: `48 | 0.9990 | 1.0000 | 237.92 ms`
- 10k TurboQuant width750 recall@100: `48 | 0.9966 | 0.9998 | 164.02 ms`
- 25k TurboQuant width750 recall@100: `48 | 0.9929 | 0.9996 | 270.48 ms`

## Matched Width750 Latency Logs

Commands:

- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg10k_g8 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --force-index --log-output review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/latency_pqg8_10k_w750_p48.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_qcmp10k_turboquant --profile ec_ivf --k 10 --iterations 100 --sweep 48 --force-index --log-output review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/latency_turbo_10k_w750_p48.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg25k_g8 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --force-index --log-output review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/latency_pqg8_25k_w750_p48.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_postopt25k_n64w25 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --force-index --log-output review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/latency_turbo_25k_w750_p48.log`

Key lines:

- 10k PQ g8 width750 p48: `48 | 100 | 88.1 ms | 8.39 ms | 80.3 ms | 85.4 ms | 104.4 ms | 117.0 ms | 135.3 ms`
- 10k TurboQuant width750 p48: `48 | 100 | 121.4 ms | 11.9 ms | 109.4 ms | 118.8 ms | 147.2 ms | 160.8 ms | 190.5 ms`
- 25k PQ g8 width750 p48: `48 | 100 | 150.6 ms | 13.0 ms | 136.4 ms | 145.7 ms | 171.9 ms | 194.1 ms | 208.1 ms`
- 25k TurboQuant width750 p48: `48 | 100 | 235.9 ms | 15.0 ms | 219.9 ms | 231.5 ms | 271.3 ms | 284.6 ms | 289.8 ms`

## restore_width25.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/restore_width25.sql --raw --log-output review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh/artifacts/restore_width25.log`

Key lines:

- `task28_ivf_pqg10k_g8_idx | ... rerank_width=25`
- `task28_ivf_pqg25k_g8_idx | ... rerank_width=25`
- `task28_ivf_qcmp10k_turboquant_idx | ... rerank_width=25`
- `task28_ivf_postopt25k_n64w25_idx | ... rerank_width=25`
