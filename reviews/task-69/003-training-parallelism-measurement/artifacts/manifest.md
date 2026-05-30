# Task 69 Packet 003 Artifact Manifest

- head SHA: `d8adfbfa51466fccfa1e6401c442283ffb368cd8`
- task bucket: `reviews/task-69/003-training-parallelism-measurement`
- timestamp: `2026-05-30T04:32:30Z`
- lane: Task 69 Slice D common-training release measurement
- fixture/storage/rerank: synthetic deterministic 10k x 1536 training vectors matching Task 68 training sample shapes; no AM storage or rerank surface
- isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `cargo-fmt-check.log`

- command: `cargo fmt --check`
- result: passed
- note: stable rustfmt prints existing warnings about unstable `imports_granularity` and `group_imports` config keys.

### `cargo-test-common-training.log`

- command: `cargo test -p ecaz --lib am::common::training --no-default-features --features pg18`
- result: passed
- key line: `test result: ok. 6 passed; 0 failed; 1 ignored; 0 measured; 1921 filtered out`

### `release-measurement-default.log`

- command: `cargo test -p ecaz --release --lib am::common::training::tests::task69_training_parallelism_measurement --no-default-features --features pg18 -- --ignored --nocapture`
- result: passed
- key lines:
  - `task69_measurement_start rayon_threads=18 rayon_env=unset`
  - `task69_measurement kind=kmeans shape=spire_10k_nlists32 rows=10000 dimensions=1536 nlists=32 iterations=8 scalar_ms=1716.896 parallel_ms=147.842 speedup=11.613 digest=59fa21d6239f0e3a parallel_digest=59fa21d6239f0e3a rayon_threads=18 rayon_env=unset`
  - `task69_measurement kind=kmeans shape=spire_100k_sample10k_nlists128 rows=10000 dimensions=1536 nlists=128 iterations=8 scalar_ms=6662.520 parallel_ms=484.940 speedup=13.739 digest=506b40bd8a9d3b8c parallel_digest=506b40bd8a9d3b8c rayon_threads=18 rayon_env=unset`
  - `task69_measurement kind=grouped_pq4 shape=ivf_pq_fastscan_10k rows=10000 dimensions=1536 group_size=16 train_size=10000 iterations=3 scalar_ms=137.797 parallel_ms=11.645 speedup=11.834 digest=facf20a7f68401d4 parallel_digest=facf20a7f68401d4 rayon_threads=18 rayon_env=unset`

### `release-measurement-rayon1.log`

- command: `RAYON_NUM_THREADS=1 cargo test -p ecaz --release --lib am::common::training::tests::task69_training_parallelism_measurement --no-default-features --features pg18 -- --ignored --nocapture`
- result: passed
- key lines:
  - `task69_measurement_start rayon_threads=1 rayon_env=1`
  - `task69_measurement kind=kmeans shape=spire_10k_nlists32 rows=10000 dimensions=1536 nlists=32 iterations=8 scalar_ms=1695.591 parallel_ms=1681.713 speedup=1.008 digest=59fa21d6239f0e3a parallel_digest=59fa21d6239f0e3a rayon_threads=1 rayon_env=1`
  - `task69_measurement kind=kmeans shape=spire_100k_sample10k_nlists128 rows=10000 dimensions=1536 nlists=128 iterations=8 scalar_ms=6656.276 parallel_ms=6633.730 speedup=1.003 digest=506b40bd8a9d3b8c parallel_digest=506b40bd8a9d3b8c rayon_threads=1 rayon_env=1`
  - `task69_measurement kind=grouped_pq4 shape=ivf_pq_fastscan_10k rows=10000 dimensions=1536 group_size=16 train_size=10000 iterations=3 scalar_ms=135.971 parallel_ms=133.147 speedup=1.021 digest=facf20a7f68401d4 parallel_digest=facf20a7f68401d4 rayon_threads=1 rayon_env=1`

### `measurement-summary.md`

- command source: manual rollup from `release-measurement-default.log` and `release-measurement-rayon1.log`
- result: records default Rayon speedups, digest equality, and single-thread regression calculation
