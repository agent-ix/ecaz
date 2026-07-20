# Direct graph reader A/B comparison

All values below come from the two cited `results.jsonl` files. The baseline is
packet 048's production persisted-head reader, which uses unprepared dynamic
SPI graph lookups. The candidate is packet 049's native heap/btree reader.

## Recall and warmed physical latency

| Scale | Recall baseline | Recall direct | Delta pp | Mean ms baseline | Mean ms direct | Delta | p95 ms baseline | p95 ms direct | Delta | p99 ms baseline | p99 ms direct | Delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 1.0000 | 1.0000 | 0.00 | 42.40 | 43.20 | +1.89% | 55.00 | 55.50 | +0.91% | 55.50 | 56.30 | +1.44% |
| 50k | 0.9800 | 0.9800 | 0.00 | 57.10 | 54.10 | -5.25% | 74.40 | 68.30 | -8.20% | 74.70 | 70.00 | -6.29% |
| 100k | 0.9500 | 0.9500 | 0.00 | 50.90 | 51.90 | +1.96% | 69.10 | 70.00 | +1.30% | 72.80 | 73.00 | +0.27% |

Every latency row is `seed_strategy=persisted_head`, `cache=warm`,
`warmup_iterations=10`, `count=50`, and `concurrency=1`.

## Same-data host control

| Scale | Single-index mean ms baseline | Direct run | Delta |
| --- | ---: | ---: | ---: |
| 10k | 2.57 | 2.85 | +10.89% |
| 50k | 3.53 | 3.29 | -6.80% |
| 100k | 3.41 | 3.46 | +1.47% |

The single-index control moves in the same direction as the physical arm at
all three scales. The physical result therefore does not demonstrate an
attributable direct-reader latency win or regression. The 50k physical
reduction is useful observed data, but the control prevents interpreting it
as a causal 5.25% improvement.

## Storage

| Scale | Physical generation bytes baseline | Direct run | Byte delta | Percent delta |
| --- | ---: | ---: | ---: | ---: |
| 10k | 242,745,344 | 242,761,728 | +16,384 | +0.006749% |
| 50k | 1,242,742,784 | 1,242,750,976 | +8,192 | +0.000659% |
| 100k | 2,496,626,688 | 2,496,626,688 | 0 | 0.000000% |

The direct reader changes scan-time access only. Two pages, one page, and zero
pages of variation across growing relations are ordinary allocation noise,
not a storage-format effect.

## Build/publish comparability

| Scale | Physical build ms baseline | Direct run | Delta | Publish ms baseline | Direct run | Delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 70,840 | 68,070 | -3.91% | 84,325 | 81,514 | -3.33% |
| 50k | 436,592 | 431,946 | -1.06% | 503,913 | 499,009 | -0.97% |
| 100k | 930,369 | 937,833 | +0.80% | 1,063,987 | 1,071,657 | +0.72% |

Build and publish times are close and also move in both directions, as
expected because the direct-reader change affects scan-time reads rather than
generation construction.

## Interpretation boundary

This matrix establishes recall and storage neutrality and rules out a large
warmed-latency regression on the measured local Intel PG18 lane. With one run
per arm and same-direction host-control movement, it does not establish a
small latency improvement. The review decision should rest on packet 049's
removal of dynamic per-hop SPI plus its correctness validation, with this
matrix treated as evidence that the structural remediation preserves measured
behavior.
