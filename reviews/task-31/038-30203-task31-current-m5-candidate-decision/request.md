# Task 31 Current M5 Candidate Decision

Reviewer: please review this refreshed top-level Task 31 decision packet.

## Scope

This packet does not execute new measurements. It stages the normalized outputs
from the current best M5 same-surface candidate reruns:

- `30196` balanced rerun on `c1a761fd` (`nprobe=96,rerank_width=500`)
- `30201` quality rerun on `886fb369` (`nprobe=96,rerank_width=1000`)

It supersedes `30200` as the current Apple-Silicon Task 31 source of truth.

## Selected Points

- balanced candidate: `ec_ivf`, `pq_fastscan`, `pq_group_size=8`,
  `nlists=128`, `nprobe=96`, `rerank_width=500`
- quality candidate: `ec_ivf`, `pq_fastscan`, `pq_group_size=8`,
  `nlists=128`, `nprobe=96`, `rerank_width=1000`

## Current Comparison

At `nprobe=96`:

| point | recall@100 | mean recall q-time | latency p50 | latency p95 | latency p99 |
|---|---:|---:|---:|---:|---:|
| balanced `w500` | `0.9676` | `10.93 ms` | `10.7 ms` | `11.6 ms` | `12.1 ms` |
| quality `w1000` | `0.9920` | `12.38 ms` | `12.1 ms` | `13.0 ms` | `13.7 ms` |

Representative explain/counter shape:

| point | execution | posting pages read | postings visited | postings scored | rerank rows |
|---|---:|---:|---:|---:|---:|
| balanced `w500` | `13.281 ms` | `1815` | `77760` | `1499` | `500` |
| quality `w1000` | `15.309 ms` | `1815` | `77760` | `2420` | `1000` |

## Why This Replaces `30200`

Compared with the previous top-level decision packet:

- balanced `w500` is unchanged
- quality `w1000` improved from `p50/p95/p99 = 12.8/13.5/13.9 ms` to
  `12.1/13.0/13.7 ms`
- quality mean recall q-time improved from `13.13 ms` to `12.38 ms`
- quality explain execution improved from `15.941 ms` to `15.309 ms`
- quality recall stayed fixed at `0.9920`
- quality scan counters stayed fixed, including `postings scored = 2420` and
  `rerank rows = 1000`

The meaningful implementation checkpoints behind that improvement were:

- `422e5ddd`: preserve score-ranked probe order
- `79c1a11c`: fetch rerank rows in heap order
- `c1a761fd`: cache rerank fetch state per scan
- `886fb369`: add Apple-Silicon NEON exact rerank inner product

## Negative Follow-On

`30202` reran the quality lane on `9203de50` after specializing the indexed
`ecvector` rerank decode path.

It did not beat `30201` cleanly:

- recall stayed fixed
- `p50` stayed flat
- `p95/p99` improved slightly
- recall mean query time regressed from `12.38 ms` to `12.66 ms`

So `30202` should be treated as a mixed or negative-result packet, not the new
baseline.

## Outcome

- The balanced point remains the low-latency recommendation.
- The quality point remains the recall-biased recommendation and still clears
  the suite thresholds:
  - recall floor: `0.992 >= 0.99`
  - p50 budget: `12.1 <= 15.0`
- `886fb369` is the current best Task 31 M5 implementation checkpoint for the
  selected quality candidate.
- This closes the current IVF optimization pass for Task 31 on Apple Silicon;
  no further generic IVF cleanup is justified by the measured results.

## Artifacts

- `artifacts/balanced-suite-manifest.json`
- `artifacts/balanced-results.jsonl`
- `artifacts/quality-suite-manifest.json`
- `artifacts/quality-results.jsonl`
- `artifacts/manifest.md`
