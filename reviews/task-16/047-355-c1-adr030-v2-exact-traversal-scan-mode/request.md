# Review Request: C1 ADR-030 V2 Exact Traversal Scan Mode

## Context

Packet `354` corrected the earlier grouped-v2 runtime narrative:

- the verified grouped lane is genuinely much faster than scalar
- but recall is far too low to be a usable drop-in operating point
- widening the live rerank window helps, but does not close the gap

Reviewer feedback on `354` narrowed the next experiment:

1. distinguish graph-structure quality from traversal-scoring quality
2. test whether the grouped-built graph can recover if traversal scoring uses
   the exact rerank payload instead of grouped approximate PQ order

That is the highest-signal runtime experiment because it keeps the same
grouped-v2 on-disk graph while changing only the scoring used to drive
candidate exploration.

## Problem

ADR-030 still did not know whether the bad verified grouped recall came mostly
from:

- a structurally weak grouped-built graph, or
- grouped approximate traversal scoring that feeds poor candidates into the
  existing rerank stage

Without a runtime mode that reuses the grouped-built graph but scores traversal
candidates exactly, the branch could not separate those two failure modes.

## Planned Slice

Batch the next tightly related slices together:

1. add an experimental grouped-v2 scan mode that uses exact rerank payload
   scoring during graph traversal
2. keep the existing grouped-v2 runtime default unchanged unless the new env
   gate is enabled
3. resolve the new gate once per `amrescan`, not per candidate, so the
   experiment does not tax its own hot path
4. add a pg proof that the exact-traversal mode emits exact comparison scores
   directly
5. remeasure representative verified `10k` and `50k` points on the real-corpus
   lane

This slice intentionally does not:

- change the on-disk grouped-v2 format
- lift either ADR-030 experimental gate
- claim a final planner-facing operating point

## Implementation

Updated:

- `src/am/scan.rs`
- `src/lib.rs`

Concrete changes:

1. added `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL`
2. resolved that gate during `amrescan` only for grouped-v2 scan descriptors
3. stored the resolved mode on `TqScanOpaque` so grouped traversal does not
   perform an env lookup on every candidate score
4. changed grouped candidate scoring so the exact-traversal mode:
   - loads the existing grouped cold rerank payload
   - scores it through the shared exact rerank path already used for grouped
     comparison output
5. left the default grouped-v2 runtime path unchanged when the new env is not
   set
6. added a pg test proving that exact-traversal grouped scans emit the same
   score they record as the comparison sidecar

## Measurements

Required checkpoint validation passed:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Scratch runtime setup for the recheck:

1. restarted scratch `pg17` with:
   - `TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD=1`
   - `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN=1`
   - `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_WINDOW=16`
   - `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL=1`
2. re-verified the existing rebuilt grouped `50k` index still executes the
   grouped lane:
   - `emitted_result_count = 40`
   - `grouped_result_count = 40`
   - `compared_result_count = 40`

Representative external-summary recheck on the final code:

### 50k grouped exact traversal vs scalar at `ef_search = 128`

Grouped exact traversal:

`tests.tqhnsw_graph_scan_recall_external_summary('tqhnsw_real_50k_corpus', 'tqhnsw_real_50k_queries_50', 'tqhnsw_real_50k_grouped_m8_idx', 8, 128)`

| path | Recall@10 | Recall@100 | NDCG@10 | mean abs score error | Spearman@10 | exact-quantized Recall@10 | graph_below_exact_queries | worst_exact_gap |
|------|-----------|------------|---------|----------------------|-------------|---------------------------|---------------------------|-----------------|
| grouped exact traversal | 0.8780 | 0.8646 | 0.9198 | 0.00566 | 0.6768 | 0.8600 | 18 | 2 |
| scalar baseline | 0.8900 | 0.8734 | 0.9289 | 0.00557 | 0.7583 | 0.8600 | 1 | 1 |

### 10k grouped exact traversal vs scalar at `ef_search = 128`

Grouped exact traversal:

`tests.tqhnsw_graph_scan_recall_external_summary('tqhnsw_real_10k_grouped_corpus', 'tqhnsw_real_10k_queries', 'tqhnsw_real_10k_grouped_m8_idx', 8, 128)`

| path | Recall@10 | Recall@100 | NDCG@10 | mean abs score error | Spearman@10 | exact-quantized Recall@10 | graph_below_exact_queries | worst_exact_gap |
|------|-----------|------------|---------|----------------------|-------------|---------------------------|---------------------------|-----------------|
| grouped exact traversal | 0.9380 | 0.9363 | 0.9607 | 0.00873 | 0.8667 | 0.9330 | 0 | 0 |
| scalar baseline | 0.9400 | 0.9363 | 0.9618 | 0.00873 | 0.8654 | 0.9310 | 0 | 0 |

I intentionally did not add a new planner-facing SQL latency claim in this
packet. The existing verified SQL launcher still expands the grouped `50k`
prefix back onto the canonical scalar corpus table, so the clean measurement
surface for this experiment remains the external recall summary lane.

## Outcome

This packet materially changes the ADR-030 diagnosis.

The grouped-built graph is not the main failure mode.

When the same grouped-v2 graph is traversed with exact rerank payload scoring:

1. `50k` recall at `ef=128` jumps from packet `354`'s grouped `0.6740` to
   `0.8780`, which is now close to scalar `0.8900`
2. `10k` recall at `ef=128` jumps from packet `354`'s grouped `0.8150` to
   `0.9380`, essentially matching scalar `0.9400`
3. the remaining gap is now small enough that the grouped candidate set looks
   recoverable, but only when traversal spends exact scoring work per candidate

That points the branch away from "grouped-v2 graph is broken" and toward the
real problem:

- grouped approximate PQ traversal scoring is too lossy for candidate selection
  at the current operating point

## Next Slice

The next runtime batch should stay on this now-narrowed question:

1. use the exact-traversal result as the upper-bound reference for grouped-v2
   candidate quality
2. target cheaper approximations to that behavior instead of widening rerank
   alone
3. add a cleaner planner-facing latency lane for grouped exact-traversal or its
   follow-on approximations once the query surface cannot silently fall back to
   the scalar canonical table
