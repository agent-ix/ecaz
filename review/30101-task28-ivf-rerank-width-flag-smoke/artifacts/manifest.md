# Artifact Manifest

Packet: `review/30101-task28-ivf-rerank-width-flag-smoke`

## recall100_pqg8_10k_p48_width1000_flag.log

- Head SHA: `803670e`
- Lane: Task 28 IVF rerank-width flag smoke
- Fixture: `task28_ivf_pqg10k_g8`, 10k corpus / 100 queries
- Storage format: `pq_fastscan`
- PQ group size: 8
- Rerank mode: `heap_f32`
- Rerank width: `1000`, via `--rerank-width 1000`
- Surface isolation: isolated one-index-per-table
- Cache state: not controlled; local warm-cache smoke
- Timestamp: `2026-04-28T10:03:10-07:00`
- Command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg10k_g8 --profile ec_ivf --k 100 --queries-limit 100 --sweep 48 --rerank-width 1000 --force-index --log-output review/30101-task28-ivf-rerank-width-flag-smoke/artifacts/recall100_pqg8_10k_p48_width1000_flag.log`
- Key result: `48 | 0.9629 | 0.9983 | 133.17 ms`

## latency10_pqg8_10k_p48_width1000_flag.log

- Head SHA: `803670e`
- Lane: Task 28 IVF rerank-width flag smoke
- Fixture: `task28_ivf_pqg10k_g8`, 10k corpus / 100 queries
- Storage format: `pq_fastscan`
- PQ group size: 8
- Rerank mode: `heap_f32`
- Rerank width: `1000`, via `--rerank-width 1000`
- Surface isolation: isolated one-index-per-table
- Cache state: not controlled; local warm-cache smoke
- Timestamp: `2026-04-28T10:03:10-07:00`
- Command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg10k_g8 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 1000 --force-index --log-output review/30101-task28-ivf-rerank-width-flag-smoke/artifacts/latency10_pqg8_10k_p48_width1000_flag.log`
- Key result: `48 | 100 | 94.7 ms | 5.67 ms | 88.9 ms | 93.7 ms | 98.1 ms | 123.3 ms | 136.0 ms`
