# Task 31 Current Candidate Decision

Reviewer: please review this refreshed top-level Task 31 decision packet.

## Scope

This packet does not execute new measurements. It stages the normalized outputs
from the current best same-revision candidate reruns:

- `30196` balanced rerun on `c1a761fd` (`nprobe=96,rerank_width=500`)
- `30195` quality rerun on `c1a761fd` (`nprobe=96,rerank_width=1000`)

It replaces the original suite-transition decision in `30185` as the
current Task 31 source of truth.

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
| quality `w1000` | `0.9920` | `13.13 ms` | `12.8 ms` | `13.5 ms` | `13.9 ms` |

Representative explain/counter shape:

| point | execution | posting pages read | postings visited | postings scored | rerank rows |
|---|---:|---:|---:|---:|---:|
| balanced `w500` | `13.281 ms` | `1815` | `77760` | `1499` | `500` |
| quality `w1000` | `15.941 ms` | `1815` | `77760` | `2420` | `1000` |

## Why This Replaces `30185`

Compared with the original suite-transition decision packet:

- balanced `w500` improved from `p50/p95/p99 = 10.9/11.8/13.3 ms` to
  `10.7/11.6/12.1 ms`
- quality `w1000` improved from `12.9/13.6/14.0 ms` to `12.8/13.5/13.9 ms`
- balanced `postings scored` dropped from `3784` to `1499`
- quality `postings scored` dropped from `6509` to `2420`
- recall stayed fixed at both selected points

The meaningful implementation checkpoints behind that improvement were:

- `422e5ddd`: preserve score-ranked probe order
- `79c1a11c`: fetch rerank rows in heap order
- `c1a761fd`: cache rerank fetch state per scan

## Negative Follow-On

`30199` reran the quality lane on `7b28c43b` after a tiny rerank-loop cleanup.
It did not beat `c1a761fd`, so `30199` should be treated as a negative-result
packet, not the new baseline.

## Outcome

- The balanced point remains the low-latency recommendation.
- The quality point remains the recall-biased recommendation and still clears
  the suite thresholds:
  - recall floor: `0.992 >= 0.99`
  - p50 budget: `12.8 <= 15.0`
- `c1a761fd` is the current best Task 31 implementation checkpoint even though
  later experimental work exists on top of it.

## Artifacts

- `artifacts/balanced-suite-manifest.json`
- `artifacts/balanced-results.jsonl`
- `artifacts/quality-suite-manifest.json`
- `artifacts/quality-results.jsonl`
- `artifacts/manifest.md`
