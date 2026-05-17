# Review Request: Task 28 IVF 25k Candidate Check

## Summary

This packet moves from the 10k anchor to a 25k DBPedia-derived local slice after
commit `81f5468`. It measures two candidates from the 100-query tuning ladder:
`32/24,width=25` and `32/32,width=25`.

No DiskANN implementation or measurement is included.

## Fixture

- corpus table: `task28_ivf_anchor25k_corpus`
- corpus rows: 25,000
- dimensions: 1536
- query table: `task28_ivf_anchor10k1536_queries100`
- exact truth: `task28_ivf_anchor25k_exact100_top10`
- queries: 100
- storage: `turboquant`
- rerank: `heap_f32`
- cache state: normal local scratch state; not cold-cache controlled

## Setup And Exact Truth

| metric | result |
|---|---:|
| table copy | `00:11.456` |
| analyze | `00:06.476` |
| exact rows | 1,000 |
| exact queries | 100 |
| exact materialization time | `18:46.749` |

The exact baseline is now the dominant local-loop cost. This should be reused
for any further 25k checks rather than recomputed.

## Candidate Results

| nlists | nprobe | width | build | materialize | recall@10 | hits | index size |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 32 | 24 | 25 | `46.138 s` | `33.660 s` | `0.9760` | 976/1000 | `22 MB` |
| 32 | 32 | 25 | `45.068 s` | `42.651 s` | `1.0000` | 1000/1000 | `22 MB` |

Latency loop:

| nlists | nprobe | width | p50 | p95 | p99 | avg |
|---:|---:|---:|---:|---:|---:|---:|
| 32 | 24 | 25 | `331.674 ms` | `371.690 ms` | `407.329 ms` | `329.108 ms` |
| 32 | 32 | 25 | `434.858 ms` | `456.380 ms` | `521.759 ms` | `429.340 ms` |

## Interpretation

At 25k rows, the full-probe local candidate still reaches exact-oracle recall
on the 100-query sample, but latency is roughly 2.4x the 10k full-probe point
from packet 30042. The `nprobe=24` candidate loses more recall at 25k
(`0.9760`) than it did at 10k (`0.9980`), so partial probing degrades as the
corpus grows.

Build time is still manageable locally at 25k (`~45s`), but exact truth
materialization at 100 queries is not a quick inner loop (`18:46.749`).

## Next Slice Recommendation

For local tuning, reuse `task28_ivf_anchor25k_exact100_top10` and test only a
small set of routing points:

- `nlists=64,nprobe=32,width=25`
- `nlists=64,nprobe=48,width=25`
- optional `nlists=32,nprobe=28,width=25`

Do not recompute exact truth unless the corpus or query set changes. A 50k local
slice should wait until the 25k routing frontier is clearer.

## Artifacts

- `artifacts/pg18-ivf-anchor25k-candidate-check.sql`
- `artifacts/pg18-ivf-anchor25k-candidate-check.log`
- `artifacts/pg18-active-during-anchor25k.log`
- `artifacts/manifest.md`

## Validation

Packet-only change.

- `git diff --check`
