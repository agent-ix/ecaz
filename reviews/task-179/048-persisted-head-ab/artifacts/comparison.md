# Persisted-head versus owner-scan comparison

All values below are parsed from the two committed `results.jsonl` files. The
baseline is the benchmark-only full-owner O(N) scan; the candidate is normal
production persisted-head seeding.

## Recall and warmed physical latency

| Scale | Recall baseline | Recall candidate | Delta pp | Mean ms baseline | Mean ms candidate | Delta | p95 ms baseline | p95 ms candidate | Delta | p99 ms baseline | p99 ms candidate | Delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 1.0000 | 1.0000 | 0.00 | 283.20 | 42.40 | -85.03% | 296.20 | 55.00 | -81.43% | 298.30 | 55.50 | -81.39% |
| 50k | 1.0000 | 0.9800 | -2.00 | 1266.60 | 57.10 | -95.49% | 1301.40 | 74.40 | -94.28% | 1317.00 | 74.70 | -94.33% |
| 100k | 0.9950 | 0.9500 | -4.50 | 2613.40 | 50.90 | -98.05% | 2663.00 | 69.10 | -97.41% | 2679.20 | 72.80 | -97.28% |

Every latency row is `cache=warm`, `warmup_iterations=10`, `count=50`, and
`concurrency=1`. Baseline rows report `seed_strategy=owner_scan`; candidate
rows report `seed_strategy=persisted_head`.

## Storage

| Scale | Physical generation bytes baseline | Candidate | Byte delta | Percent delta |
| --- | ---: | ---: | ---: | ---: |
| 10k | 242,745,344 | 242,745,344 | 0 | 0.000000% |
| 50k | 1,242,734,592 | 1,242,742,784 | +8,192 | +0.000659% |
| 100k | 2,496,626,688 | 2,496,626,688 | 0 | 0.000000% |

The feature changes only scan-time seed acquisition. Both builds persist the
same bounded head data. The isolated one-page 50k difference is ordinary
relation allocation variance, not a storage-format change.

## Same-data host control

| Scale | Single-index mean ms baseline | Candidate | Delta |
| --- | ---: | ---: | ---: |
| 10k | 2.46 | 2.57 | +4.5% |
| 50k | 3.90 | 3.53 | -9.5% |
| 100k | 3.41 | 3.41 | 0.0% |

The control movement is small relative to the physical arm's 85-98% mean
reduction and is not consistently favorable to the candidate.

## Build/publish comparability

| Scale | Physical build ms baseline | Candidate | Publish ms baseline | Candidate |
| --- | ---: | ---: | ---: | ---: |
| 10k | 68,194 | 70,840 | 81,568 | 84,325 |
| 50k | 433,872 | 436,592 | 502,300 | 503,913 |
| 100k | 929,622 | 930,369 | 1,064,950 | 1,063,987 |

The build/publish times track closely, as expected: the benchmark feature does
not change generation construction or persisted storage.

## Interpretation boundary

This matrix establishes the cost of the removed scan on otherwise-current
code and satisfies the requested evidence comparison. It does not establish
recall neutrality. The persisted-head candidate meets the configured 0.90
floor and reproduces the previously reviewed cap-4096 outcome, but loses 2.0
and 4.5 recall points relative to full-owner seeding at 50k and 100k. Acceptance
of that bounded-work tradeoff requires the requested outside-review decision.
