# Task 28 IVF A9 Current Size and Cache State

## Scope

This packet records the current-head storage and cache-state note for the selected 100k IVF operating point after A7 score-bound pruning.

Fixture:

- prefix: `task28_ivf_pqg100k_g8_n128`
- quantizer: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- `rerank=heap_f32`
- `rerank_width=500`

## Result

| field | value |
|---|---:|
| index size | 19,791,872 bytes |
| index size pretty | 19 MB |
| corpus heap size | 7,675,904 bytes |
| corpus total size | 1,686,732,800 bytes |
| cache state | warm local development run; no explicit cache drop |

The index reloptions in the raw output confirm the expected surface: `nlists=128`, `storage_format=pq_fastscan`, `pq_group_size=8`, `rerank=heap_f32`, and `rerank_width=500`.

## A9 Current Selected Point

The current selected 100k IVF point now has packet-local or cited raw evidence for:

- build time: `216414.112 ms`, from packet 30092's `build_g8_100k_n128_surface.log`
- index size: `19,791,872` bytes, from this packet
- recall@10: `0.9920`, from packet 30116
- recall@100: `0.9552`, from packet 30117
- latency p50/p95/p99: `173.1/204.9/210.5 ms`, from packet 30116
- memory HWM: `156692 kB`, from packet 30116
- cache state: warm local run, no explicit cache drop, from this packet

## Validation

- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30118-task28-ivf-a9-current-size-cache/artifacts/ivf_100k_n128_size_snapshot.sql --raw --log-output review/30118-task28-ivf-a9-current-size-cache/artifacts/ivf_100k_n128_size_snapshot.log`

## Next

A9 is close for the IVF-selected 100k operating point. The remaining decision is whether to require a fresh current-head rebuild log or accept the existing packet 30092 build artifact for this unchanged 100k n128 surface.
