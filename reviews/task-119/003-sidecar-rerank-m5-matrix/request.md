# Task 119: M5 Sidecar Rerank Matrix Evidence

## Summary

This packet adds M5-local benchmark evidence for the core Task 119 matrix:

```text
HNSW RaBitQ 1-bit candidate frontier + second-stage rerank representation
```

The measured variants are exactly:

- `f32`
- `rabitq2`, `rabitq4`, `rabitq8`
- `turboquant_2bit`, `turboquant_3bit`, `turboquant_4bit`,
  `turboquant_5bit`, `turboquant_6bit`, `turboquant_7bit`,
  `turboquant_8bit`

All measurements use `ecaz bench suite`, 10k/50k/100k real corpora,
`ef_search={320,500,1000}`, `candidate_k=1000`, `queries_limit=200`,
and `read_mode=free`.

## Artifacts

- Manifest: `reviews/task-119/003-sidecar-rerank-m5-matrix/artifacts/manifest.md`
- Suite config: `crates/ecaz-cli/suites/task119-hnsw-rabitq-sidecar-rerank-matrix.json`
- 10k results: `reviews/task-119/003-sidecar-rerank-m5-matrix/artifacts/suite-results.10k.jsonl`
- 50k results: `reviews/task-119/003-sidecar-rerank-m5-matrix/artifacts/suite-results.50k.jsonl`
- 100k results: `reviews/task-119/003-sidecar-rerank-m5-matrix/artifacts/suite-results.100k.jsonl`
- Full table logs:
  - `reviews/task-119/003-sidecar-rerank-m5-matrix/artifacts/sidecar-10k-hnsw-rabitq-required-rerank-matrix.log`
  - `reviews/task-119/003-sidecar-rerank-m5-matrix/artifacts/sidecar-50k-hnsw-rabitq-required-rerank-matrix.log`
  - `reviews/task-119/003-sidecar-rerank-m5-matrix/artifacts/sidecar-100k-hnsw-rabitq-required-rerank-matrix.log`

Each JSONL result file has 33 rows: 11 rerank variants x 3 `ef_search` values.

## Key `ef_search=1000` Results

| Scale | Variant | Recall@10 | NDCG@10 | candidate SQL p50 | sidecar score p50 | total bound p50 | bytes/vector | sidecar size |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | `f32` | 0.9765 | 0.9956 | 14.616 ms | 22.342 ms | 36.953 ms | 6144 | 58.59 MiB |
| 10k | `rabitq8` | 0.9650 | 0.9955 | 14.616 ms | 9.542 ms | 24.147 ms | 1548 | 14.76 MiB |
| 10k | `turboquant_4bit` | 0.9535 | 0.9953 | 14.616 ms | 10.185 ms | 24.798 ms | 772 | 7.36 MiB |
| 10k | `turboquant_8bit` | 0.9730 | 0.9956 | 14.616 ms | 83.705 ms | 98.403 ms | 1540 | 14.69 MiB |
| 50k | `f32` | 0.9885 | 0.9993 | 17.236 ms | 22.451 ms | 39.795 ms | 6144 | 292.97 MiB |
| 50k | `rabitq8` | 0.9475 | 0.9989 | 17.236 ms | 9.620 ms | 26.922 ms | 1548 | 73.81 MiB |
| 50k | `turboquant_4bit` | 0.9390 | 0.9989 | 17.236 ms | 10.340 ms | 27.593 ms | 772 | 36.81 MiB |
| 50k | `turboquant_8bit` | 0.9790 | 0.9992 | 17.236 ms | 85.923 ms | 103.297 ms | 1540 | 73.43 MiB |
| 100k | `f32` | 0.9850 | 0.9993 | 26.498 ms | 22.609 ms | 49.171 ms | 6144 | 585.94 MiB |
| 100k | `rabitq8` | 0.9420 | 0.9990 | 26.498 ms | 9.635 ms | 36.134 ms | 1548 | 147.63 MiB |
| 100k | `turboquant_4bit` | 0.9415 | 0.9990 | 26.498 ms | 10.053 ms | 36.586 ms | 772 | 73.62 MiB |
| 100k | `turboquant_8bit` | 0.9760 | 0.9993 | 26.498 ms | 84.678 ms | 111.113 ms | 1540 | 146.87 MiB |

The manifest contains the full `ef_search=1000` table for all variants; the
JSONL artifacts contain all `ef_search=320`, `500`, and `1000` rows with
p50/p95/p99 metrics.

## Outcome

This matrix supports keeping Task 119 experimental and iterating, not promoting
a production HNSW RaBitQ coarse-rerank profile yet.

Findings:

- `f32` rerank confirms that a stronger second-stage representation can recover
  the most recall, but it is not the storage-saving answer: 6144 bytes/vector
  and 585.94 MiB sidecar size at 100k.
- `rabitq8` is the fastest measured scoring lane, but at 100k it reaches only
  0.9420 recall@10 at `ef_search=1000`, 4.3 points below `f32`.
- `turboquant_4bit` is the best compact Pareto lane in this harness: at 100k it
  gets 0.9415 recall@10 with 772 bytes/vector and 36.586 ms total-bound p50.
  This is the 1536-dimensional special no-QJL lane.
- QJL-active `turboquant_7bit` and `turboquant_8bit` approach `f32` recall but
  are much slower than `f32`, `rabitq8`, and `turboquant_4bit` in this sidecar
  implementation.
- `turboquant_2bit` and `turboquant_3bit` are not viable here: both are slower
  than the compact RaBitQ lanes and lose much more recall.

## Caveats

This packet is not a full Task 119 closeout. It covers the required
rerank-representation matrix at 10k/50k/100k on the M5 host, but the harness
still does not report every acceptance-counter field:

- no visited graph-node counter;
- no heap/source read counter under production sidecar I/O;
- no build-time measurement in this packet because it reuses existing isolated
  task119 corpus/index tables;
- no production sidecar read mode, because these runs use `read_mode=free`.

The correct next step is to keep Task 119 open and either extend the suite
runner/counters or run a follow-up production-I/O packet for the best candidate
lanes (`f32`, `rabitq8`, `turboquant_4bit`, and possibly `turboquant_8bit` for
recall ceiling context).
