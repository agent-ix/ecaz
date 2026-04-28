# Artifact Manifest

Head SHA: `1a05603b2425fb74cb06362479ed71f2ce56ea46`

Packet: `review/30084-task28-ivf-quantizer-headtohead-smoke`

Timestamp: 2026-04-27 21:00 America/Los_Angeles

Fixture:

- Source prefix: `task28_ivf_postopt10k_n64w25`
- Derived prefixes:
  - `task28_ivf_qcmp10k_turboquant`
  - `task28_ivf_qcmp10k_pqfastscan`
  - `task28_ivf_qcmp10k_rabitq`
- Rows: 10000 corpus rows, 100 query rows
- Dimensions: 1536
- Isolated one-index-per-table surfaces: yes
- Storage formats: `turboquant`, `pq_fastscan`, `rabitq`
- Rerank mode: `heap_f32`
- Rerank width: 25
- nlists: 64
- Runtime nprobe sweep: 32, 48 except narrowed RaBitQ latency smoke
- Cache state: warm local development run; no explicit cache drop
- Memory high-water mark: not captured

## build_quantizer_surfaces.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30084-task28-ivf-quantizer-headtohead-smoke/artifacts/build_quantizer_surfaces.sql --raw --log-output review/30084-task28-ivf-quantizer-headtohead-smoke/artifacts/build_quantizer_surfaces.log`

Key lines:

- TurboQuant build: `CREATE INDEX` in `21722.014 ms`
- PQ-FastScan build: `CREATE INDEX` in `24761.363 ms`
- RaBitQ build: `CREATE INDEX` in `22143.177 ms`
- `task28_ivf_qcmp10k_pqfastscan_idx | 1968 kB`
- `task28_ivf_qcmp10k_rabitq_idx | 9416 kB`
- `task28_ivf_qcmp10k_turboquant_idx | 9416 kB`

## recall_turboquant.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp10k_turboquant --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30084-task28-ivf-quantizer-headtohead-smoke/artifacts/recall_turboquant.log`

Key lines:

- `32 | 0.9800 | 0.9981 | 61.99 ms`
- `48 | 1.0000 | 1.0000 | 83.49 ms`

## recall_pqfastscan.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp10k_pqfastscan --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30084-task28-ivf-quantizer-headtohead-smoke/artifacts/recall_pqfastscan.log`

Key lines:

- `32 | 0.3880 | 0.9079 | 32.94 ms`
- `48 | 0.3890 | 0.9081 | 39.48 ms`

## recall_rabitq.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp10k_rabitq --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30084-task28-ivf-quantizer-headtohead-smoke/artifacts/recall_rabitq.log`

Key lines:

- `32 | 0.9800 | 0.9981 | 1219.21 ms`
- `48 | 1.0000 | 1.0000 | 1846.27 ms`

## latency_turboquant.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_qcmp10k_turboquant --profile ec_ivf --k 10 --iterations 100 --sweep 32,48 --force-index --log-output review/30084-task28-ivf-quantizer-headtohead-smoke/artifacts/latency_turboquant.log`

Key lines:

- `32 | 100 | 63.6 ms | ... | p50 63.1 ms | p95 69.8 ms | p99 76.2 ms`
- `48 | 100 | 83.6 ms | ... | p50 82.6 ms | p95 89.8 ms | p99 94.3 ms`

## latency_pqfastscan.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_qcmp10k_pqfastscan --profile ec_ivf --k 10 --iterations 100 --sweep 32,48 --force-index --log-output review/30084-task28-ivf-quantizer-headtohead-smoke/artifacts/latency_pqfastscan.log`

Key lines:

- `32 | 100 | 33.1 ms | ... | p50 32.7 ms | p95 34.5 ms | p99 36.8 ms`
- `48 | 100 | 41.1 ms | ... | p50 40.2 ms | p95 46.3 ms | p99 50.7 ms`

## latency_rabitq_narrow.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_qcmp10k_rabitq --profile ec_ivf --k 10 --iterations 10 --sweep 32 --force-index --log-output review/30084-task28-ivf-quantizer-headtohead-smoke/artifacts/latency_rabitq_narrow.log`

Key lines:

- `32 | 10 | 1292.7 ms | ... | p50 1276.7 ms | p95 1407.3 ms | p99 1428.2 ms`
