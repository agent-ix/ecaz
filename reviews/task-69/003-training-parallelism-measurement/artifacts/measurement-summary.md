# Task 69 Training Parallelism Measurement Summary

## Scope

- Code commit: `d8adfbfa51466fccfa1e6401c442283ffb368cd8`
- Harness: ignored release-mode unit test
  `am::common::training::tests::task69_training_parallelism_measurement`
- Command shape: `cargo test -p ecaz --release --lib ... -- --ignored --nocapture`
- Synthetic deterministic vectors: 10,000 rows, 1,536 dimensions, seed 69
- Byte-equality gate: each measured parallel result is asserted equal to the
  scalar baseline before the timing line is printed.

## Default Rayon Pool

| kind | shape | scalar_ms | parallel_ms | speedup | digest | parallel_digest | rayon_threads |
| --- | --- | ---: | ---: | ---: | --- | --- | ---: |
| kmeans | `spire_10k_nlists32` | 1716.896 | 147.842 | 11.613x | `59fa21d6239f0e3a` | `59fa21d6239f0e3a` | 18 |
| kmeans | `spire_100k_sample10k_nlists128` | 6662.520 | 484.940 | 13.739x | `506b40bd8a9d3b8c` | `506b40bd8a9d3b8c` | 18 |
| grouped_pq4 | `ivf_pq_fastscan_10k` | 137.797 | 11.645 | 11.834x | `facf20a7f68401d4` | `facf20a7f68401d4` | 18 |

## RAYON_NUM_THREADS=1

| kind | shape | scalar_ms | parallel_ms | parallel/scalar | regression |
| --- | --- | ---: | ---: | ---: | ---: |
| kmeans | `spire_10k_nlists32` | 1695.591 | 1681.713 | 0.9918 | -0.8% |
| kmeans | `spire_100k_sample10k_nlists128` | 6656.276 | 6633.730 | 0.9966 | -0.3% |
| grouped_pq4 | `ivf_pq_fastscan_10k` | 135.971 | 133.147 | 0.9792 | -2.1% |

Worst observed single-thread regression: none. The slowest
`RAYON_NUM_THREADS=1` parallel path was still slightly faster than the scalar
baseline in this run.
