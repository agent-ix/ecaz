# Review Request: C1 ADR-030 V2 Real-50k M16 Runtime Validation

## Context

Packet `361` stabilized the grouped-v2 build surface by making HNSW graph
construction deterministic. After that fix, the canonical `50k` grouped `m=8`
lane stopped wandering and settled at:

- grouped `m=8`, `window=64`
  - `ef=64`: `0.904 Recall@10 @ 1.177ms`
  - `ef=128`: `0.910 Recall@10 @ 1.601ms`
  - `ef=320`: `0.914 Recall@10 @ 3.561ms`

The obvious next question was whether the same stabilized binary-scored grouped
runtime improves further at `m=16`, which is the more typical HNSW default in
database systems.

## Problem

The branch had a strong `m=8` direct result, but no current canonical `50k`
`m=16` rerun on the stabilized deterministic-build lane. Without that, the
branch could not answer:

> does the grouped-v2 binary runtime remain competitive when the graph density
> moves to `m=16`, and how does it compare to scalar on the same canonical
> `50k` surface?

## Planned Slice

This batch is measurement-only. No code changes.

1. keep the current deterministic grouped build from packet `361`
2. keep grouped runtime settings unchanged:
   - grouped scan gate enabled
   - grouped build gate enabled
   - `grouped_scan_window = 64`
   - `grouped_scan_score_mode = binary`
3. build fresh canonical `50k` scalar and grouped indexes at `m=16`
4. rerun the same direct recall/latency sweep used for the `m=8` readout

## Implementation

No repository code changes in this packet.

Scratch-only build actions:

1. built canonical scalar `m=16`:
   - `tqhnsw_real_50k_m16_idx`
2. built canonical grouped `m=16`:
   - `tqhnsw_real_50k_grouped_m16_idx`

Both were created on the same canonical corpus table:

- `tqhnsw_real_50k_corpus`

using:

- `m = 16`
- `ef_construction = 128`

and for the grouped build:

- `build_source_column = 'source'`

## Validation

Scratch runtime settings were verified before the rerun:

- grouped build gate enabled
- grouped scan gate enabled
- `grouped_scan_window = 64`
- `grouped_scan_score_mode = binary`

Direct measurement command shape:

- `tests.tqhnsw_graph_scan_recall_ef_sweep(...)`
- query subset: `tqhnsw_real_50k_queries_50`
- `ef_search`: `40,64,100,128,160,200,256,320`

## Measurements

### Canonical `50k` grouped `m=16`

| ef_search | Recall@10 | exact-quantized Recall@10 | mean query latency ms |
|----------:|----------:|--------------------------:|----------------------:|
| 40  | `0.900` | `0.944` | `1.155` |
| 64  | `0.930` | `0.944` | `1.302` |
| 100 | `0.936` | `0.944` | `1.961` |
| 128 | `0.936` | `0.944` | `2.445` |
| 160 | `0.936` | `0.944` | `2.951` |
| 200 | `0.936` | `0.944` | `3.756` |
| 256 | `0.936` | `0.944` | `4.581` |
| 320 | `0.938` | `0.944` | `6.132` |

### Canonical `50k` scalar `m=16`

| ef_search | Recall@10 | exact-quantized Recall@10 | mean query latency ms |
|----------:|----------:|--------------------------:|----------------------:|
| 40  | `0.944` | `0.944` | `1.903` |
| 64  | `0.952` | `0.944` | `2.899` |
| 100 | `0.950` | `0.944` | `3.909` |
| 128 | `0.950` | `0.944` | `5.183` |
| 160 | `0.950` | `0.944` | `5.876` |
| 200 | `0.950` | `0.944` | `7.363` |
| 256 | `0.956` | `0.944` | `8.261` |
| 320 | `0.956` | `0.944` | `9.987` |

### Readout

The grouped `m=16` lane is clearly stronger than grouped `m=8`, but it does
not catch scalar `m=16` on recall:

- grouped `m=16` reaches `0.900` at `ef=40` in `1.155ms`
- grouped `m=16` reaches `0.930` at `ef=64` in `1.302ms`
- grouped `m=16` tops out around `0.938` by `ef=320`
- scalar `m=16` starts at `0.944` already at `ef=40`

So the trade-off is:

- grouped `m=16` remains materially faster than scalar `m=16`
- scalar `m=16` remains materially more accurate than grouped `m=16`

Representative same-`ef` comparison:

- grouped `m=16`, `ef=128`: `0.936 @ 2.445ms`
- scalar `m=16`, `ef=128`: `0.950 @ 5.183ms`

Representative “crossed target” comparison:

- grouped `m=16` never reaches scalar `m=16`'s low-end `0.944` operating point
  within the measured `ef<=320` sweep
- grouped `m=16` does, however, beat scalar `m=16` decisively on latency for
  the entire measured range

## Risk / Follow-up

The `m=16` rerun changes the decision boundary:

1. grouped-v2 binary runtime is still viable and fast at `m=16`
2. but the best measured grouped `m=16` recall still trails scalar `m=16`
3. so the next useful work is not “does `m=16` help?”; it already does
4. the next useful work is:
   - planner-facing SQL measurement on the stabilized `m=16` lane
   - or a deliberate product decision that grouped is a latency-first mode, not
     a scalar-replacement-at-equal-recall mode

