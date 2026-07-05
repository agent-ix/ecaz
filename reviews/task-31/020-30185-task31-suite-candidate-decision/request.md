# Task 31 Suite Candidate Decision

Reviewer: please review this decision packet comparing the suite-runner
balanced and quality candidates.

## Scope

This packet does not execute new measurements. It stages the normalized outputs
from:

- `30184` balanced candidate suite run (`nprobe=96,rerank_width=500`)
- `30183` quality candidate suite run (`nprobe=96,rerank_width=1000`)

The goal is to leave one packet-local comparison that selects the two Task 31
100k `n128` decision points after the suite-runner transition.

## Decision

Selected points:

- balanced candidate: `ec_ivf`, `pq_fastscan`, `pq_group_size=8`,
  `nlists=128`, `nprobe=96`, `rerank_width=500`
- quality candidate: `ec_ivf`, `pq_fastscan`, `pq_group_size=8`,
  `nlists=128`, `nprobe=96`, `rerank_width=1000`

## Comparison

At `nprobe=96`:

| point | recall@100 | mean recall q-time | latency p50 | latency p95 | latency p99 |
|---|---:|---:|---:|---:|---:|
| balanced `w500` | `0.9676` | `11.21 ms` | `10.9 ms` | `11.8 ms` | `13.3 ms` |
| quality `w1000` | `0.9920` | `13.35 ms` | `12.9 ms` | `13.6 ms` | `14.0 ms` |

Representative explain/counter shape:

| point | execution | posting pages read | postings visited | postings scored | rerank rows |
|---|---:|---:|---:|---:|---:|
| balanced `w500` | `14.143 ms` | `1815` | `77760` | `3784` | `500` |
| quality `w1000` | `17.087 ms` | `1815` | `77760` | `6509` | `1000` |

The fixed scan volume is the important stability signal. Both points walk the
same centroid/list space; the quality point pays almost entirely for the wider
rerank window.

## Outcome

- The balanced point remains the default-like local recommendation because it
  holds the lower latency profile while keeping recall@10 at the already-high
  `0.9980` level established in earlier Task 31 packets.
- The quality point remains the quality-biased recommendation because it is the
  only suite-runner point here that clears recall@100 `0.99`.
- `30183`'s suite thresholds passed for the quality point:
  - recall floor: `0.992 >= 0.99`
  - p50 budget: `12.9 <= 15.0`

## Artifacts

- `artifacts/balanced-suite-manifest.json`
- `artifacts/balanced-results.jsonl`
- `artifacts/quality-suite-manifest.json`
- `artifacts/quality-results.jsonl`
- `artifacts/manifest.md`
