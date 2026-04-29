# Task 29 DiskANN prior-neighbor prune probe

## Status

This packet measures commit `e70fc267b9a93a572366e8be068a88154ec6506c`, which
fixes the Vamana build candidate pool so each node's second-pass
`RobustPrune` considers both the fresh greedy-search visited set and that
node's existing out-neighbors.

The change is algorithmically correct and should stay, but it does not remove
the Task 29 landing blocker. On real-10k it gives a small low-`list_size`
quality/runtime improvement, adds about 8% build time, and does not raise the
recall ceiling.

## Validation

Code validation before commit `e70fc267`:

```text
cargo test --lib am::ec_diskann::vamana
7 passed

cargo test --lib am::ec_diskann
159 passed

cargo pgrx test pg18 pg_test_ec_diskann_
19 passed

cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings
passed

git diff --check
passed
```

PG18 extension install for measurement:

```text
cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18
```

## Setup

Prefix:

```text
task29_diskann_prior_real10k
```

Reloptions:

```text
profile=ec_diskann
graph_degree=32
build_list_size=100
alpha=1.2
```

Corpus:

```text
target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv
target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv
rows=10000
queries=200
dim=1536
bits=4
seed=42
```

The source TSV sibling manifest still names the original prepared prefix
`ec_hnsw_real_10k`; this isolated load therefore used the checked-in loader's
explicit `--allow-manifest-mismatch` flag.

## Correctness Of Measurement Path

`artifacts/explain-diskann-prior-q1.log` confirms the rebuilt table uses the
DiskANN index at `list_size=64`:

```text
Index Scan using task29_diskann_prior_real10k_idx on task29_diskann_prior_real10k_corpus
Buffers: shared hit=922
Execution Time: 58.636 ms
```

No cache flush was performed; these are local warm-cache-after-build/recall
measurements.

## Load And Build Timing

Prior-neighbor fix (`artifacts/load-diskann-prior.log`):

```text
copied corpus table task29_diskann_prior_real10k_corpus in 9.80s
encoded corpus table task29_diskann_prior_real10k_corpus in 4.43s
copied queries table task29_diskann_prior_real10k_queries in 218.14ms
built task29_diskann_prior_real10k_idx in 530.42s
completed prefix task29_diskann_prior_real10k in 561.40s
```

Baseline alpha=1.2 from packet 676 built in `491.05s` and completed in
`520.66s`; the fix makes this local build about 8% slower.

## Recall Sweep

Prior-neighbor fix (`artifacts/recall-diskann-prior-table.log`):

| list_size | recall@10 | NDCG@10 | mean query |
| ---: | ---: | ---: | ---: |
| 64 | 0.9320 | 0.9967 | 59.51 ms |
| 128 | 0.9310 | 0.9967 | 69.06 ms |
| 200 | 0.9315 | 0.9966 | 82.99 ms |
| 400 | 0.9315 | 0.9966 | 130.58 ms |
| 800 | 0.9315 | 0.9966 | 284.48 ms |

Baseline alpha=1.2 from packet 676:

| list_size | recall@10 | NDCG@10 | mean query |
| ---: | ---: | ---: | ---: |
| 64 | 0.9280 | 0.9959 | 70.96 ms |
| 128 | 0.9310 | 0.9966 | 74.01 ms |
| 200 | 0.9315 | 0.9966 | 84.70 ms |
| 400 | 0.9315 | 0.9966 | 126.73 ms |
| 800 | 0.9315 | 0.9966 | 268.90 ms |

## Latency And Memory

Prior-neighbor fix, 200 iterations per point, concurrency 1
(`artifacts/latency-diskann-prior-table.log`):

| list_size | mean | p50 | p95 | p99 | HWM |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 64 | 60.2 ms | 59.7 ms | 65.6 ms | 76.5 ms | 123592 KiB |
| 128 | 73.1 ms | 71.9 ms | 88.3 ms | 101.7 ms | 124232 KiB |
| 200 | 83.6 ms | 82.9 ms | 93.5 ms | 108.2 ms | 123432 KiB |
| 400 | 127.5 ms | 127.4 ms | 140.8 ms | 147.6 ms | 123912 KiB |
| 800 | 279.5 ms | 277.4 ms | 314.1 ms | 348.4 ms | 124232 KiB |

The latency shape is broadly similar to packet 676. Some means are lower, but
the p99 and sampled HWM are higher in this run; do not treat this as a decisive
latency win without repeated runs.

## Storage

Prior-neighbor fix (`artifacts/storage-diskann-prior.log`):

```text
task29_diskann_prior_real10k_idx  ec_diskann  {graph_degree=32,build_list_size=100,alpha=1.2}  4.7 MiB  494.0 B/row
total relation size: 164.5 MiB
```

Storage is unchanged from the alpha=1.2 baseline.

## Recommendation

Keep `e70fc267`: preserving prior out-neighbors during per-node `RobustPrune`
matches Vamana's intended candidate pool and has no correctness downside in
the focused PG18 tests.

Do not treat it as the landing fix. The recall ceiling remains around
`0.9315`, still below the HNSW m16 reference in packet 676 (`0.9700` at the
comparable 200 setting), while build time is still much slower than HNSW.

Next optimization/landing blocker: add a graph-shape diagnostic for real-10k
DiskANN builds, then inspect degree distribution, medoid reachability, and
greedy-search candidate pool sizes. The benchmark evidence now points to graph
navigability/candidate generation, not scan `list_size`, `alpha`, or simple
prior-neighbor preservation.
