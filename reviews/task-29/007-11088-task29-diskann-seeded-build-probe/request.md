# Review Request: Task 29 seeded DiskANN build probe

Branch: `task29-diskann-initial-tuning`
Author: coder1

## What This Packet Tests

Packet `11087` showed that the persisted DiskANN graph is clean and reachable but
has extreme in-degree hubbing. I tested a conservative build-side optimization:
seed the in-memory Vamana graph with deterministic random out-neighbors before
the existing two robust-prune passes.

Measured commit: `2fb991ff` (`Seed DiskANN Vamana build graph`)

Result: negative. The code was reverted in `55700d47` after measurement.

## Validation

Before measuring the seeded build code:

- `cargo test --lib am::ec_diskann::vamana`
- `cargo test --lib am::ec_diskann`
- `cargo check --all-targets --no-default-features --features pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`
- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18`

After reverting the seeded build code:

- `cargo test --lib am::ec_diskann::vamana`
- `git diff --check HEAD~1..HEAD`

## Measurement

Fresh isolated real-10k prefix:

```text
task29_diskann_seed_real10k
profile=ec_diskann
graph_degree=32
build_list_size=100
alpha=1.2
```

Build/load (`artifacts/load-diskann-seeded.log`):

- copy corpus: `9.83s`
- encode corpus: `4.95s`
- copy queries: `213.92ms`
- build index: `523.93s`
- total load: `555.60s`

Graph diagnostics (`artifacts/graph-diskann-seeded.log`):

- live reachable: `10000 / 10000`
- no dead, invalid, self, duplicate, or unresolvable neighbor refs
- out degree avg `24.5015`, min `6`, p50 `24`, p95 `32`, max `32`
- in degree avg `24.5015`, min `4`, p50 `22`, p95 `43`, p99 `61`, max `3480`

Recall (`artifacts/recall-diskann-seeded-table.log`):

| list_size | recall@10 | NDCG | mean q-time |
|---:|---:|---:|---:|
| 64 | 0.9315 | 0.9967 | 60.26 ms |
| 128 | 0.9310 | 0.9967 | 71.04 ms |
| 200 | 0.9315 | 0.9966 | 84.90 ms |
| 400 | 0.9315 | 0.9966 | 133.17 ms |
| 800 | 0.9315 | 0.9966 | 278.34 ms |

Latency (`artifacts/latency-diskann-seeded-table.log`):

| list_size | mean | p50 | p95 | p99 | HWM |
|---:|---:|---:|---:|---:|---:|
| 64 | 61.1 ms | 59.5 ms | 75.1 ms | 83.4 ms | 118152 KiB |
| 128 | 68.7 ms | 68.7 ms | 73.7 ms | 76.5 ms | 118632 KiB |
| 200 | 82.2 ms | 82.2 ms | 89.4 ms | 91.0 ms | 118952 KiB |
| 400 | 127.8 ms | 127.7 ms | 143.7 ms | 161.4 ms | 118632 KiB |
| 800 | 277.1 ms | 276.6 ms | 303.2 ms | 313.1 ms | 118632 KiB |

Storage (`artifacts/storage-diskann-seeded.log`):

- DiskANN index size: `4.7 MiB`
- per row: `494.0 B/row`

## Comparison

Against packet `678` prior-neighbor probe:

- recall ceiling is unchanged (`0.9315`)
- seeded graph has similar total edge count (`245015` vs `245035`)
- seeded graph keeps full reachability, but max in-degree is worse
  (`3480` vs `3250`)
- build time is essentially the same class (`523.93s` vs `530.42s`) and still
  far slower than the HNSW reference row from packet `676`

## Recommendation

Do not keep deterministic random graph seeding as the first DiskANN landing
optimization. It does not move recall, latency, storage, or hub concentration
enough to justify adding it.

Next useful slice: instrument build-time candidate generation itself. The
evidence now rules out simple scan width, alpha, prior-neighbor preservation,
and random initial out-neighbors. The remaining likely blocker is the candidate
pool/prune interaction during build, especially whether greedy search is
presenting a diverse enough pool before `robust_prune`.
