# Review Request: Task 28 IVF Anchor Nprobe Debug

## Summary

This packet follows up on packet 30035's suspicious `0.71` flat recall result.
The issue was the fixture, not `ec_ivf.nprobe`: packet 30035 used an older
10k corpus with 64-dimensional `source` vectors. This packet switches to the
DBPedia anchor corpus, where `source` is 1536-dimensional, and records EXPLAIN
evidence for each `nprobe` point.

No DiskANN implementation or measurement is included.

## Fixture Check

`artifacts/pg18-anchor-dimension-check.log` records:

| relation | rows | source dim |
|---|---:|---:|
| `ec_hnsw_real_ann_benchmarks_anchor_corpus` | 990,000 | 1536 |
| `ec_hnsw_real_ann_benchmarks_anchor_queries` | 10,000 | 1536 |
| `ec_hnsw_parallel_concurrent_dsm_recall_corpus` | 10,000 | 64 |

So the previous `0.71` smoke should not be read as a real DBPedia IVF quality
baseline.

## Attempted 50k Local Build

I first tried a copied 50k x 1536 DBPedia subset with `nlists = 128` and
`training_sample_rows = 50000`. The table copy completed, but `CREATE INDEX`
was still active after roughly 9 minutes and did not observe normal
`pg_cancel_backend` / `pg_terminate_backend` promptly while CPU-bound in the
build. I killed the single scratch backend to unblock the session.

This is not a benchmark number, but it is a useful local-tuning signal:
full-sample spherical k-means at 50k x 1536 is too heavy for the first local
iteration loop.

## Successful Nprobe Debug

The successful run uses a smaller but still DBPedia-derived 1536-dimensional
slice:

- source: `ec_hnsw_real_ann_benchmarks_anchor_corpus`
- corpus rows: 10,000
- query rows: 20
- dimensions: 1536
- index: `ec_ivf`, `nlists = 32`, `training_sample_rows = 2000`
- storage: `turboquant`
- rerank: `off`
- cache state: normal local scratch state; not cold-cache controlled

Build/storage:

| metric | result |
|---|---:|
| build time | `00:24.934` |
| index size | `9,379,840` bytes (`9160 kB`) |
| heap size | `1,048,576` bytes (`1024 kB`) |

EXPLAIN confirms `ec_ivf.nprobe` is honored:

| nprobe | selected lists | posting pages read | candidates scored | one-query execution time |
|---:|---:|---:|---:|---:|
| 1 | 1 | 26 | 223 | `14.888 ms` |
| 4 | 4 | 130 | 1,137 | `28.572 ms` |
| 16 | 16 | 678 | 6,016 | `108.394 ms` |
| 32 | 32 | 1,112 | 10,000 | `171.477 ms` |

Recall against the packet-local exact compressed scoring table:

| nprobe | returned | exact hits | recall@10 |
|---:|---:|---:|---:|
| 1 | 200 | 88 | 0.4400 |
| 4 | 200 | 134 | 0.6700 |
| 16 | 200 | 175 | 0.8750 |
| 32 | 200 | 184 | 0.9200 |

Every query returned 10 rows at every `nprobe`.

## Interpretation

The nprobe path works on the real 1536-dimensional anchor: selected lists,
candidate count, latency, and recall all scale in the expected direction.

The remaining concern is full-probe recall. With `nprobe = nlists = 32`, the
scan scores all 10,000 indexed candidates but only matches 0.92 recall@10
against the SQL exact compressed-scoring baseline. That may be an expected
compressed-only scorer mismatch, but it should be checked before treating
full-probe IVF as an exact oracle. The next slice should compare the AM scorer
and SQL operator scorer on the same candidate set.

## Artifacts

- `artifacts/pg18-anchor-dimension-check.log`
- `artifacts/pg18-ivf-anchor50k-n128-nprobe-debug.sql`
- `artifacts/pg18-ivf-anchor50k-n128-nprobe-debug.log`
- `artifacts/pg18-active-while-50k.log`
- `artifacts/pg18-cancel-heavy-50k-build.log`
- `artifacts/pg18-active-after-cancel.log`
- `artifacts/pg18-terminate-heavy-50k-build.log`
- `artifacts/pg18-active-after-terminate.log`
- `artifacts/pg18-ivf-anchor10k1536-n32-nprobe-debug.sql`
- `artifacts/pg18-ivf-anchor10k1536-n32-nprobe-debug.log`
- `artifacts/pg18-active-anchor10k1536.log`
- `artifacts/manifest.md`

## Validation

Packet-only change; no code changed.

- `git diff --check`

## Next Slice Recommendation

Before broad `nlists` sweeps, run a scorer-alignment packet:

1. For the 10k x 1536 / `nlists=32` fixture, materialize the full-probe IVF
   output and exact SQL output for the same queries.
2. Compare score ordering and score values for disagreements.
3. Decide whether full-probe `rerank=off` should be documented as approximate
   or whether the AM scorer should be brought into parity with the SQL
   compressed operator.
