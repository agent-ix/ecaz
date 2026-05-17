# Task 31 Rerank State Cache Decision

Reviewer: please review this Task 31 decision packet after rerunning both
candidate lanes on the IVF rerank-state-cache checkpoint `c1a761fd`.

## Scope

This packet does not execute new measurements. It stages the normalized outputs
from:

- `30195` quality rerun (`nprobe=96,rerank_width=1000`)
- `30196` balanced rerun (`nprobe=96,rerank_width=500`)

## Summary

The rerank-state cache is a smaller follow-up than the heap-order fetch change,
but it still looks worth keeping:

- quality `w1000` improved tail latency and explain time with recall unchanged
- balanced `w500` was mixed in the latency table but kept the same recall and
  counter shape, with explain back down to `13.281 ms`

At `nprobe=96`:

| point | recall@100 | latency p50 | latency p95 | latency p99 | explain exec | postings scored | rerank rows |
|---|---:|---:|---:|---:|---:|---:|---:|
| balanced `w500` | `0.9676` | `10.7 ms` | `11.6 ms` | `12.1 ms` | `13.281 ms` | `1499` | `500` |
| quality `w1000` | `0.9920` | `12.8 ms` | `13.5 ms` | `13.9 ms` | `15.941 ms` | `2420` | `1000` |

## Decision

- Keep `c1a761fd` as the current Task 31 implementation checkpoint.
- Preserve the same Task 31 decision points:
  - balanced candidate: `ec_ivf`, `pq_fastscan`, `pq_group_size=8`,
    `nlists=128`, `nprobe=96`, `rerank_width=500`
  - quality candidate: `ec_ivf`, `pq_fastscan`, `pq_group_size=8`,
    `nlists=128`, `nprobe=96`, `rerank_width=1000`
- Treat this as an incremental quality-lane cleanup on top of `79c1a11c`,
  not as a large new phase change.

## Next Checkpoint

The next slice should move beyond setup cleanup and attack the remaining exact
rerank cost directly, or else switch from implementation work to a refreshed
Task 31 top-level decision packet using `30195` and `30196`.

## Artifacts

- `artifacts/balanced-suite-manifest.json`
- `artifacts/balanced-results.jsonl`
- `artifacts/quality-suite-manifest.json`
- `artifacts/quality-results.jsonl`
- `artifacts/manifest.md`
