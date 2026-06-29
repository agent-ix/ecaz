# Task 124 Packet 006 Artifact Manifest

- head SHA: `2744aa4f2c0290ce2b44f20ae1177518e416fab5`
- task bucket: `reviews/task-124/006-tq-no-qjl-gamma-elision`
- timestamp: `2026-06-29T02:38:39Z`
- lane: local PG18 release build on `/Users/peter/.pgrx`, database `tqvector_bench`
- fixture: `data/staged-current/ec_real_{10k,50k,100k}_{corpus,queries}.tsv`
- runner: `ecaz bench suite`
- isolation: one index per table/prefix

## Artifact Inventory

- config: `artifacts/task124-tq-no-qjl-gamma-elision-final15-ab-suite.json`
- run log: `artifacts/suite-run-r2.log`
- suite manifest: `artifacts/final15-ab-suite-r2/suite-manifest.json`
- suite results: `artifacts/final15-ab-suite-r2/results.jsonl`
- per-step load, recall, latency, and storage logs under `artifacts/final15-ab-suite-r2/`

Generated truth caches are intentionally untracked and should not be committed.

## Command

```text
/Users/peter/.cargo/bin/ecaz bench suite run --config reviews/task-124/006-tq-no-qjl-gamma-elision/artifacts/task124-tq-no-qjl-gamma-elision-final15-ab-suite.json --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/006-tq-no-qjl-gamma-elision/artifacts/suite-run-r2.log
```

## Code Change Under Test

The slice removes the stored `f32` gamma from no-QJL TurboQuant rerank sidecar
payloads. For the measured 1536D lane, TQ sidecar payload length changes from
`4 + code_len` to `code_len`, where `code_len = 768` bytes. QJL-active TQ lanes
still store gamma because the QJL scoring path consumes it.

## A/B Recall

| scale | f32/source nprobe32 | TQ final15 nprobe32 | f32/source nprobe64 | TQ final15 nprobe64 |
| --- | ---: | ---: | ---: | ---: |
| 10k | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 50k | 0.9960 | 0.9960 | 1.0000 | 0.9990 |
| 100k | 0.9730 | 0.9730 | 1.0000 | 1.0000 |

## A/B Latency

| scale | variant | nprobe32 p50 | nprobe32 p95 | nprobe32 p99 | nprobe64 p50 | nprobe64 p95 | nprobe64 p99 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | f32/source | 0.80 ms | 0.91 ms | 1.02 ms | 1.29 ms | 1.41 ms | 1.49 ms |
| 10k | TQ final15 | 0.76 ms | 0.86 ms | 1.08 ms | 1.23 ms | 1.43 ms | 1.71 ms |
| 50k | f32/source | 2.47 ms | 2.67 ms | 2.82 ms | 4.87 ms | 5.04 ms | 5.15 ms |
| 50k | TQ final15 | 2.43 ms | 2.68 ms | 2.89 ms | 4.83 ms | 4.98 ms | 5.09 ms |
| 100k | f32/source | 5.13 ms | 5.57 ms | 5.83 ms | 9.23 ms | 9.39 ms | 9.55 ms |
| 100k | TQ final15 | 5.14 ms | 5.72 ms | 6.11 ms | 9.30 ms | 9.55 ms | 9.71 ms |

## A/B Storage

| scale | f32/source index | TQ final15 index |
| --- | ---: | ---: |
| 10k | 2.9 MiB / 305.6 B per row | 10.9 MiB / 1140.3 B per row |
| 50k | 11.6 MiB / 243.3 B per row | 50.8 MiB / 1064.8 B per row |
| 100k | 22.5 MiB / 235.8 B per row | 100.8 MiB / 1056.6 B per row |

## SIMD Counters

All TQ latency points in this packet report:

- `isa=neon`
- `scalar_candidates=0`
- `width_ge32=100`
- 10,000 TQ candidates per latency sweep point

## Interpretation

This is a safe TQ payload cleanup, but it does not close Task 124.

- Recall is unchanged from packet 005, including the 50k/nprobe64 TQ drop from
  1.0000 to 0.9990.
- Latency is not a durable win: 100k TQ is slightly worse than f32/source at both
  measured nprobes in this run.
- Storage remains roughly 4x f32/source because the 1536D no-QJL 4-bit TQ code
  still costs 768 bytes before page/layout overhead.

The next TQ optimization should target a materially smaller stage-2 payload or
payload locality/layout. The measured scorer path is already full SIMD.
