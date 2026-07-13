# Prompt cancellation A/B comparison

All values below come from the two cited `results.jsonl` files. The baseline is
packet 050's direct physical graph reader before async PostgreSQL interrupt
polling. The candidate adds packet 051's 5 ms local-cancel poll and bounded
cancel-token delivery.

## Recall and warmed physical latency

| Scale | Recall baseline | Recall prompt poll | Delta pp | Mean ms baseline | Mean ms prompt poll | Delta | p95 ms baseline | p95 ms prompt poll | Delta | p99 ms baseline | p99 ms prompt poll | Delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 1.0000 | 1.0000 | 0.00 | 43.20 | 43.50 | +0.69% | 55.50 | 55.70 | +0.36% | 56.30 | 56.10 | -0.36% |
| 50k | 0.9800 | 0.9800 | 0.00 | 54.10 | 54.50 | +0.74% | 68.30 | 67.90 | -0.59% | 70.00 | 72.30 | +3.29% |
| 100k | 0.9500 | 0.9500 | 0.00 | 51.90 | 49.50 | -4.62% | 70.00 | 67.40 | -3.71% | 73.00 | 75.90 | +3.97% |

Every latency row is `seed_strategy=persisted_head`, `cache=warm`,
`warmup_iterations=10`, `count=50`, and `concurrency=1`.

## Same-data host control

| Scale | Single-index mean ms baseline | Prompt-poll run | Delta |
| --- | ---: | ---: | ---: |
| 10k | 2.85 | 2.83 | -0.70% |
| 50k | 3.29 | 3.38 | +2.74% |
| 100k | 3.46 | 3.56 | +2.89% |

Neither the physical arm nor the host control moves consistently across all
scales. At 100k the physical mean/p95 improve while p99 and the host control
increase. The matrix therefore shows no material attributable poll overhead;
it does not establish a causal latency improvement.

## Storage

| Scale | Physical generation bytes baseline | Prompt-poll run | Byte delta | Percent delta |
| --- | ---: | ---: | ---: | ---: |
| 10k | 242,761,728 | 242,761,728 | 0 | 0.000000% |
| 50k | 1,242,750,976 | 1,242,734,592 | -16,384 | -0.001318% |
| 100k | 2,496,626,688 | 2,496,634,880 | +8,192 | +0.000328% |

The change affects wait/cancel behavior only. Zero, two, and one pages of
variation across these relations are ordinary allocation noise, not a
storage-format effect.

## Build/publish comparability

| Scale | Physical build ms baseline | Prompt-poll run | Delta | Publish ms baseline | Prompt-poll run | Delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 68,070 | 67,807 | -0.39% | 81,514 | 81,240 | -0.34% |
| 50k | 431,946 | 430,834 | -0.26% | 499,009 | 498,004 | -0.20% |
| 100k | 937,833 | 923,253 | -1.55% | 1,071,657 | 1,057,833 | -1.29% |

Build and publish times remain close. Their small favorable movement cannot be
caused by a scan-time interrupt poll that is inactive during generation.

## Interpretation boundary

This matrix establishes recall and storage neutrality and finds no material
warmed-latency overhead from prompt cancellation on the measured local Intel
PG18 lane. With one run per arm and mixed mean/tail/control movement, it cannot
resolve sub-millisecond effects or support a speedup claim. The closeout
decision should combine this measured neutrality with packet 051's live proof
that both mid-await and mid-connect cancellation now unwind in under one second.
