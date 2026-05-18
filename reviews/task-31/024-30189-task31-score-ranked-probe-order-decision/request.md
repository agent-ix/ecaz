# Task 31 Score-Ranked Probe Order Decision

Reviewer: please review this Task 31 follow-up decision packet after the
balanced and quality reruns on the score-ranked probe-order checkpoint.

## Scope

This packet does not execute new measurements. It stages the normalized outputs
from:

- `30187` quality rerun (`nprobe=96,rerank_width=1000`)
- `30188` balanced rerun (`nprobe=96,rerank_width=500`)

The goal is to summarize whether `422e5ddd` is a worthwhile checkpoint before
the next rerank-focused implementation slice.

## Summary

The probe-order change is a real counter-shape win on both candidate lanes.

At `nprobe=96`:

| point | recall@100 | latency p50 | explain exec | postings visited | postings scored | rerank rows |
|---|---:|---:|---:|---:|---:|---:|
| balanced `w500` | `0.9676` | `10.7 ms` | `13.281 ms` | `77760` | `1499` | `500` |
| quality `w1000` | `0.9920` | `13.1 ms` | `16.070 ms` | `77760` | `2420` | `1000` |

Compared with the pre-change packets:

- balanced `w500` reduced `postings scored` from `3784` to `1499`
- quality `w1000` reduced `postings scored` from `6509` to `2420`
- recall stayed fixed on both points
- the balanced latency table improved modestly
- the quality latency table stayed roughly flat/noisy despite the better
  pruning counters

## Decision

- Keep `422e5ddd` as a valid implementation checkpoint. It improves
  PQ-FastScan bound-pruning effectiveness without regressing recall or suite
  thresholds.
- Do not treat it as a complete Task 31 performance win yet. The balanced lane
  got the expected latency improvement; the quality lane did not show the same
  clean benchmark response.
- Preserve the same Task 31 decision points:
  - balanced candidate: `ec_ivf`, `pq_fastscan`, `pq_group_size=8`,
    `nlists=128`, `nprobe=96`, `rerank_width=500`
  - quality candidate: `ec_ivf`, `pq_fastscan`, `pq_group_size=8`,
    `nlists=128`, `nprobe=96`, `rerank_width=1000`

## Next Checkpoint

The next slice should stay on rerank cost:

- either run repeatability packets on `422e5ddd` to classify the quality-lane
  latency noise
- or implement the next heap-f32 rerank reduction now that the scan is handing
  fewer candidates into rerank

## Artifacts

- `artifacts/balanced-suite-manifest.json`
- `artifacts/balanced-results.jsonl`
- `artifacts/quality-suite-manifest.json`
- `artifacts/quality-results.jsonl`
- `artifacts/manifest.md`

