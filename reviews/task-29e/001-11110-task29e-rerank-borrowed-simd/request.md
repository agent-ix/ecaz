# Task 29e Rerank / Scratch Follow-up

## Status

Small code cleanup landed in `009d433c`:

- exact heap rerank now scores a borrowed `ecvector` datum slice instead of
  copying the heap vector into a fresh `Vec<f32>`;
- the scan rerank dot product reuses the same AVX2/FMA-dispatched inner-product
  helper as build.

This closes the obvious code-level discrepancy with pgvectorscale's full-distance
rescore path, but it does **not** materially close the latency gap on the
isolated `ec_diskann` surface. The measurement supports the reviewer note from
`11109`: heap row fetch / tuple-slot work dominates over the f32 dot product.

## Isolated Measurement

Compared with `11109` final isolated latency, the rerank cleanup is essentially
neutral:

| L | `11109` mean | `009d433c` mean | decision |
|---|---:|---:|---|
| 64 | 7.80 ms | 7.70 ms | tiny / noise |
| 128 | 7.79 ms | 7.76 ms | tiny / noise |
| 200 | 7.98 ms | 8.10 ms | tiny / noise |
| 400 | 8.49 ms | 8.60 ms | tiny / noise |
| 800 | 9.34 ms | 9.33 ms | tiny / noise |

Recall stayed in family:

| L | recall@10 | NDCG@10 |
|---|---:|---:|
| 64 | 0.9965 | 0.9999 |
| 128 | 0.9965 | 0.9999 |
| 200 | 0.9970 | 0.9999 |
| 400 | 0.9970 | 0.9999 |
| 800 | 0.9975 | 0.9999 |

## Rejected Experiments

Two local A/Bs were not kept:

- Scan neighbor-list consume: avoided cloning the expanded node's neighbor list,
  but isolated latency regressed at higher L (`L=400` mean `9.18 ms`,
  `L=800` mean `9.83 ms`).
- Build epoch-mark scratch: removed repeated build-side mark-vector allocation,
  but real-10k rebuild regressed to `15.155 s` total versus the `11109`
  `14.59 s` baseline.

## Recommendation

Keep `009d433c` as a small symmetry/cleanup patch, not as a performance claim.
Do not pursue the two rejected local experiments for landing. The remaining
meaningful gap is structural: low-L latency needs fewer heap visits or a
different exact-rerank storage path; build parity likely needs a larger graph
construction/layout change rather than scratch allocation tweaks.
