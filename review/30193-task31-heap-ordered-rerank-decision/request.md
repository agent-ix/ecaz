# Task 31 Heap-Ordered Rerank Decision

Reviewer: please review this Task 31 decision packet after rerunning both
candidate lanes on the heap-ordered rerank-fetch checkpoint `79c1a11c`.

## Scope

This packet does not execute new measurements. It stages the normalized outputs
from:

- `30191` quality rerun (`nprobe=96,rerank_width=1000`)
- `30192` balanced rerun (`nprobe=96,rerank_width=500`)

The goal is to decide whether the heap-ordered rerank fetch should be kept as
the current Task 31 measurement checkpoint.

## Summary

The heap-ordered rerank fetch preserves both Task 31 decision points and
improves latency on both lanes without changing recall or scan counters.

At `nprobe=96`:

| point | recall@100 | latency p50 | latency p95 | latency p99 | explain exec | postings scored | rerank rows |
|---|---:|---:|---:|---:|---:|---:|---:|
| balanced `w500` | `0.9676` | `10.6 ms` | `11.3 ms` | `11.5 ms` | `13.931 ms` | `1499` | `500` |
| quality `w1000` | `0.9920` | `12.8 ms` | `13.6 ms` | `14.1 ms` | `16.330 ms` | `2420` | `1000` |

Compared with the score-ranked-probe-only packets:

- balanced `w500` improved latency from `10.7/11.4/11.9 ms` to
  `10.6/11.3/11.5 ms`
- quality `w1000` improved latency from `13.1/14.2/15.1 ms` to
  `12.8/13.6/14.1 ms`
- recall stayed fixed on both points
- postings scored and other scan counters stayed fixed on both points, which is
  exactly what this rerank-only change should do

## Decision

- Keep `79c1a11c` as the current Task 31 implementation checkpoint.
- Preserve the same Task 31 decision points:
  - balanced candidate: `ec_ivf`, `pq_fastscan`, `pq_group_size=8`,
    `nlists=128`, `nprobe=96`, `rerank_width=500`
  - quality candidate: `ec_ivf`, `pq_fastscan`, `pq_group_size=8`,
    `nlists=128`, `nprobe=96`, `rerank_width=1000`
- Treat this as a real end-to-end improvement over `422e5ddd`: the earlier
  scan-side pruning win is now paired with cleaner rerank-heavy latency.

## Next Checkpoint

The next slice should stay on rerank cost or result staging:

- either move into the next heap-f32 rerank reduction
- or restage the higher-level Task 31 candidate-decision packet using `30192`
  and `30191` as the new lane sources of truth

## Artifacts

- `artifacts/balanced-suite-manifest.json`
- `artifacts/balanced-results.jsonl`
- `artifacts/quality-suite-manifest.json`
- `artifacts/quality-results.jsonl`
- `artifacts/manifest.md`
