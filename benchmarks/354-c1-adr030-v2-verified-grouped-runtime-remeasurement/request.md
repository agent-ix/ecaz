# Review Request: C1 ADR-030 V2 Verified Grouped Runtime Remeasurement

## Context

Packet `352` staged the first `1k/10k/50k` grouped runtime measurements.
Packet `353` made the live grouped rerank window configurable and rechecked the
`50k` lane at `window = 8`.

While following packet `353`'s next step, I used the grouped debug window
summary against the scratch real-corpus indexes and found a more important
problem:

- the supposed scratch `grouped` indexes were not actually grouped-v2 on disk
- `tests.tqhnsw_debug_grouped_scan_windowed_summary(...)` reported
  `grouped_result_count = 0` on those indexes
- the old `grouped` lanes were therefore measuring fresh scalar rebuilds under
  grouped names, not real grouped-v2 runtime behavior

That invalidates the earlier scratch grouped conclusions in packet `352`.

## Problem

ADR-030 did not just need a wider window experiment; it needed a verified
grouped-v2 measurement lane first.

Until the scratch grouped indexes were rebuilt under a postmaster that was
definitely started with:

- `TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD=1`
- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN=1`

the branch had no trustworthy grouped runtime numbers on real corpus data.

## Planned Slice

Batch the next measurement-repair slices together:

1. verify whether the scratch `grouped` indexes were actually grouped-v2
2. restart the scratch postmaster with the ADR-030 build/scan gates
3. rebuild the real-corpus grouped indexes on that verified server
4. re-run grouped-v2 measurements on `50k` and `10k`
5. use the grouped window diagnostics to determine whether rerank width is still
   a material lever on the real grouped lane

This packet is measurement-only:

- no production code changes
- no gate-lift claims
- no attempt to preserve the stale grouped scratch indexes

## Measurement Setup

Scratch verification steps:

1. manually registered `tests.tqhnsw_debug_grouped_scan_windowed_summary(...)`
   in the scratch database from the installed extension SQL so the loaded
   real-corpus fixtures could use the existing grouped diagnostic surface
   without dropping and recreating the extension
2. verified that the old scratch grouped indexes reported
   `grouped_result_count = 0`, which means scalar metadata
3. restarted scratch `pg17` with:
   - `TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD=1`
   - `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN=1`
   - `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_WINDOW=8`, then later `16`
4. rebuilt:
   - `tqhnsw_real_50k_grouped_m8_idx` on `tqhnsw_real_50k_corpus`
   - `tqhnsw_real_10k_grouped_m8_idx` on `tqhnsw_real_10k_grouped_corpus`

Format verification after rebuild:

- `tqhnsw_real_50k_grouped_m8_idx` first-query grouped window summary:
  - `emitted_result_count = 40`
  - `grouped_result_count = 40`
  - `compared_result_count = 40`
- `tqhnsw_real_10k_grouped_m8_idx` first-query grouped window summary:
  - `emitted_result_count = 40`
  - `grouped_result_count = 40`
  - `compared_result_count = 40`

Those rebuilt indexes are the first scratch real-corpus lanes in this branch
that were confirmed to be real grouped-v2 runtime execution.

## Results

### 50k emitted-set window evidence on the verified grouped index

Aggregated over `tqhnsw_real_50k_queries_50` using
`tests.tqhnsw_debug_grouped_scan_windowed_summary(...)`:

| window | exact-best at rank 1 | exact-top4 fully inside rerank window | mean abs rank shift after | max abs rank shift after | mean Spearman after |
|--------|----------------------|---------------------------------------|---------------------------|--------------------------|---------------------|
| 1 | 0.4400 | 0.0200 | 8.6000 | 36 | 0.5262 |
| 4 | 0.8200 | 0.1800 | 6.1080 | 33 | 0.7346 |
| 8 | 0.8400 | 0.3200 | 3.7800 | 29 | 0.8694 |
| 16 | 0.9200 | 0.5400 | 1.1760 | 21 | 0.9729 |

Interpretation:

- the real grouped lane is not saturated at `window = 8`
- `window = 16` still buys a large improvement inside the emitted set
- rerank width is still a material runtime lever on the verified grouped path

### 50k live grouped-v2 direct-harness sweeps

Verified grouped-v2, live `window = 8`:

| ef_search | grouped Recall@10 | exact-quantized Recall@10 | grouped mean latency ms |
|-----------|-------------------|---------------------------|-------------------------|
| 40 | 0.5820 | 0.8600 | 0.9785 |
| 64 | 0.6100 | 0.8600 | 1.0338 |
| 100 | 0.6060 | 0.8600 | 1.4046 |
| 128 | 0.5980 | 0.8600 | 1.7245 |
| 160 | 0.6020 | 0.8600 | 2.1183 |
| 200 | 0.6060 | 0.8600 | 2.8101 |

Verified grouped-v2, live `window = 16`:

| ef_search | grouped Recall@10 | exact-quantized Recall@10 | grouped mean latency ms |
|-----------|-------------------|---------------------------|-------------------------|
| 40 | 0.6580 | 0.8600 | 0.9355 |
| 64 | 0.6800 | 0.8600 | 1.0383 |
| 100 | 0.6760 | 0.8600 | 1.4372 |
| 128 | 0.6740 | 0.8600 | 1.7127 |
| 160 | 0.6780 | 0.8600 | 2.0738 |
| 200 | 0.6820 | 0.8600 | 2.5713 |

Same-cluster scalar baseline:

| ef_search | scalar Recall@10 | exact-quantized Recall@10 | scalar mean latency ms |
|-----------|------------------|---------------------------|------------------------|
| 40 | 0.8600 | 0.8600 | 1.4011 |
| 64 | 0.8760 | 0.8600 | 1.8094 |
| 100 | 0.8840 | 0.8600 | 2.4593 |
| 128 | 0.8900 | 0.8600 | 3.0425 |
| 160 | 0.8920 | 0.8600 | 3.7502 |
| 200 | 0.8940 | 0.8600 | 4.5234 |

Interpretation:

- the verified grouped-v2 `50k` lane is much faster than scalar
- but it is far below scalar recall
- widening the live window from `8` to `16` materially improves recall with
  only a small latency change
- even at `window = 16`, grouped-v2 is still not close to scalar quality on
  `50k`

### 10k live grouped-v2 direct-harness sweeps

Verified grouped-v2, live `window = 16`:

| ef_search | grouped Recall@10 | exact-quantized Recall@10 | grouped mean latency ms |
|-----------|-------------------|---------------------------|-------------------------|
| 40 | 0.7965 | 0.7965 | 0.5597 |
| 64 | 0.8100 | 0.7965 | 0.7930 |
| 100 | 0.8150 | 0.7965 | 1.1452 |
| 128 | 0.8150 | 0.7965 | 1.3222 |
| 160 | 0.8150 | 0.7965 | 1.5912 |
| 200 | 0.8150 | 0.7965 | 1.7731 |

Same-cluster scalar baseline:

| ef_search | scalar Recall@10 | exact-quantized Recall@10 | scalar mean latency ms |
|-----------|------------------|---------------------------|------------------------|
| 40 | 0.9310 | 0.9310 | 2.6841 |
| 64 | 0.9335 | 0.9310 | 3.6300 |
| 100 | 0.9385 | 0.9310 | 4.9175 |
| 128 | 0.9400 | 0.9310 | 5.9704 |
| 160 | 0.9400 | 0.9310 | 6.9593 |
| 200 | 0.9400 | 0.9310 | 8.8769 |

Interpretation:

- the earlier "10k grouped is directionally good" conclusion from packet `352`
  does not survive verification
- the verified grouped-v2 `10k` lane is again much faster, but far below scalar
  recall

## Outcome

The important outcome is not a new operating-point win. It is a corrected
understanding of the branch:

1. packet `352`'s scratch grouped numbers were not measuring real grouped-v2
   runtime
2. once the grouped lane is verified on-disk, grouped-v2 is substantially
   faster but substantially less accurate than scalar on both `10k` and `50k`
3. live rerank width still matters on the verified grouped lane:
   `window = 16` is materially better than `8`
4. but rerank width alone is not enough to close the grouped-v2 quality gap

This narrows the next ADR-030 question sharply:

- not "is grouped-v2 already competitive?"
- but "what is wrong with the candidate set / approximate ordering feeding the
  rerank stage, and how much more can a wider rerank prefix recover before that
  deeper problem dominates?"

## Next Slice

The next runtime batch should stay on the verified grouped lane and target the
quality gap directly:

1. decide whether the `window <= 16` cap is still artificially constraining the
   live runtime, since the emitted-set diagnostics are still improving at `16`
2. inspect the real grouped traversal pipeline for what is still missing from
   the intended `binary -> grouped -> rerank` shape
3. re-run any future corpus measurements only after verifying the scratch index
   format first, so grouped-v2 evidence cannot silently regress back to scalar
