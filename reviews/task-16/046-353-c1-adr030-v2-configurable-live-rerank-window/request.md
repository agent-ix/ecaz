# Review Request: C1 ADR-030 V2 Configurable Live Rerank Window

## Context

Packet `351` cut over grouped-v2 live scans to a real rerank window with a
fixed width of `4` behind `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN`.

Packet `352` then staged the first real runtime validation at `1k`, `10k`, and
`50k`:

- `10k` looked directionally good
- `50k` did not
- the strongest next-step suggestion was to test whether the live rerank
  prefix was simply too narrow before treating the larger recall gap as a
  deeper structural problem

Reviewer feedback on `351` and `352` converged on the same seam:

1. the live rerank window should become tunable instead of staying hardcoded at
   `4`
2. the first follow-up experiment should be a `50k` window-width recheck

## Problem

With the live grouped-v2 window fixed at `4`, the branch had no way to test
whether the `50k` gap was caused by:

- a rerank prefix that was too narrow, or
- approximate grouped ordering that was already too noisy before rerank

Until the live window was configurable, any window-width comparison would have
required a code edit and rebuild for every candidate operating point.

## Planned Slice

Batch the next tightly related runtime slices together:

1. make the grouped-v2 live rerank window configurable behind the existing scan
   gate
2. validate the new runtime control surface with focused pg coverage
3. re-run the first `50k` grouped operating-point check at a wider live window
   (`8`) before broadening SQL benchmarking again

This packet intentionally does not:

- lift either ADR-030 experimental gate
- claim that a wider window fixes the `50k` recall gap
- widen the full planner benchmark matrix beyond one same-cluster grouped/scalar
  comparison pass

## Implementation

Updated:

- `src/am/scan.rs`
- `src/lib.rs`

Concrete changes:

1. replaced the hardcoded grouped live rerank window with:
   - default width `4`
   - env override `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_WINDOW`
   - accepted range `1..=16`
2. resolved the grouped live window during `amrescan` only for grouped-v2 scan
   descriptors, while scalar scans keep the inert default
3. sized the inline grouped live-rerank buffer to the maximum supported width
   and used the resolved per-scan window for refill and overflow invariants
4. added pg coverage proving:
   - invalid window values are rejected during grouped scan startup
   - the default runtime window still matches the `window_size = 4`
     simulation
   - an env-configured live `window = 8` runtime matches the
     `window_size = 8` simulation on a real grouped query

## Measurements

Required checkpoint validation passed:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Focused validation also passed:

- `cargo test test_grouped_v2_runtime_live_window_matches_windowed_simulation -- --nocapture`
- `cargo test test_grouped_v2_runtime_live_window_respects_window_env -- --nocapture`
- `cargo test test_grouped_v2_runtime_rejects_invalid_live_window_env -- --nocapture`

After installing the checkpoint into the scratch cluster and restarting `pg17`
with:

- `TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD=1`
- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN=1`
- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_WINDOW=8`

I re-ran the `50k` grouped recall sweep:

`tests.tqhnsw_graph_scan_recall_ef_sweep('tqhnsw_real_50k_corpus', 'tqhnsw_real_50k_queries_50', 'tqhnsw_real_50k_grouped_m8_idx', 8, array[40,64,100,128,160,200])`

| ef_search | grouped Recall@10 | exact-quantized Recall@10 | grouped mean latency ms |
|-----------|-------------------|---------------------------|-------------------------|
| 40 | 0.8560 | 0.8560 | 1.5449 |
| 64 | 0.8620 | 0.8560 | 1.9868 |
| 100 | 0.8680 | 0.8560 | 2.7957 |
| 128 | 0.8700 | 0.8560 | 3.2719 |
| 160 | 0.8740 | 0.8560 | 3.7323 |
| 200 | 0.8760 | 0.8560 | 4.3904 |

Compared with packet `352`'s live `window = 4` results:

- recall is unchanged across the sweep
- grouped direct-harness latency is modestly better at every sampled `ef`
- widening the live rerank window alone does not close the `50k` recall gap

Same-cluster planner-facing SQL latency on `50k` at `query_limit = 50`:

| ef_search | grouped `window=8` mean ms | scalar mean ms |
|-----------|----------------------------|----------------|
| 40 | 4.695 | 4.726 |
| 64 | 5.431 | 5.519 |
| 100 | 7.037 | 6.430 |
| 128 | 7.375 | 7.416 |
| 160 | 8.219 | 8.100 |
| 200 | 8.891 | 9.343 |

Interpretation:

- the wider live window slightly improves the grouped SQL latency shape at
  `50k`
- grouped is now near parity or slightly better at `ef = 40/64/128/200`
- grouped still trails scalar at `ef = 100/160`
- the main ADR-030 blocker remains quality, not this specific runtime knob

## Outcome

ADR-030 now has a real live-window control surface for grouped-v2 scans, and
the first `50k` recheck says something important:

- the live `window = 4` choice was not the whole problem
- widening to `8` is safe and measurable
- widening alone does not recover the missing `50k` recall

That is a useful narrowing result. It shifts the next runtime investigation away
from "just make the rerank prefix larger" and toward the quality of the
approximate candidate order feeding that rerank step.

## Next Slice

The next runtime batch should target the `50k` quality gap more directly:

1. measure whether the emitted-set simulation saturates by `window = 8/16` on
   the same corpus, so we can separate "rerank-width limit" from
   "approximate-order limit"
2. inspect the actual grouped scan pipeline for what is still missing from the
   intended `binary -> grouped -> rerank` shape
3. re-measure the `50k` grouped lane only after that narrower runtime change,
   instead of expanding the broader SQL bench matrix again
