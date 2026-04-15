# Review Request: C1 ADR-030 V2 Selective Exact Traversal Budget

## Context

Packet `355` established the key runtime diagnosis:

- the grouped-built graph is not the main failure
- grouped approximate traversal scoring is the main reason verified grouped-v2
  recall collapses
- all-layer exact traversal on the same grouped-built graph recovers recall close
  to scalar, but at far too much cost

That left the next question narrow and concrete:

1. can a cheaper selective-exact traversal mode preserve most or all of packet
   `355`'s recall recovery
2. if so, where is the real cost floor for that family of approaches

Reviewer feedback on `354` and `355` pointed at two exact-like follow-ons before
changing the format again:

- exact scoring on only part of the traversal
- bounded exact rescoring of only the candidates that matter most

## Problem

The branch still did not know whether exact traversal needed to score:

- every grouped candidate on every enabled layer, or
- only a narrow exact subset per expansion

Without that separation, ADR-030 could not tell whether the "exact traversal
rescues recall" result was a viable approximation target or only a quality
upper bound with no realistic cost path.

The scratch workflow also still depended on ad hoc restart commands for the
ADR-030 env matrix, which made repeated runtime experiments noisy and fragile.

## Planned Slice

Batch the next tightly related slices together:

1. keep the exact-traversal gate resolved once per `amrescan`
2. add an exact-traversal scope so the branch can test `layer0`-only rescue
3. add a per-expansion exact candidate budget on enabled grouped traversal
   layers
4. pick budgeted exact candidates using grouped approximate score order, then
   leave the rest on grouped approximate scores
5. add a reusable scratch restart wrapper so repeated ADR-030 runtime runs stop
   depending on one-off shell env strings
6. remeasure verified `10k` and `50k` grouped indexes on:
   - `layer0` exact traversal
   - all-layer budgeted exact traversal

This slice intentionally does not:

- claim a planner-facing operating point
- change the grouped-v2 on-disk format
- lift any ADR-030 experimental gate

## Implementation

Updated:

- `src/am/scan.rs`
- `src/lib.rs`
- `scripts/restart_adr030_scratch.sh`

Concrete changes:

1. kept `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL` resolved once
   per `amrescan`
2. added
   `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_SCOPE`
   with:
   - default exact behavior = `all`
   - explicit selective mode = `layer0`
3. added
   `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_LIMIT`
   as a bounded per-expansion exact budget on enabled grouped traversal layers
4. changed grouped candidate collection so budgeted exact traversal:
   - computes grouped approximate scores first
   - exact-rescores only the best `N` grouped candidates per expansion
   - leaves the remaining grouped candidates on grouped approximate scores
5. kept exact-vs-approx selection local to grouped traversal scoring without
   changing scalar paths
6. added unit coverage for:
   - scope gating by layer
   - exact budget activation only on enabled layers
   - stable exact-candidate selection order
7. added pg coverage rejecting invalid exact-traversal limit env values
8. added `scripts/restart_adr030_scratch.sh` so the scratch runtime can be
   restarted with explicit `window/scope/limit` settings through one repo-local
   command that prints the active ADR-030 env surface before startup

## Validation

Required checkpoint validation passed on the final code:

- `cargo test`
- `/bin/bash -lc 'PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17'`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Focused checks also passed while landing the slice:

- `cargo test grouped_exact_traversal`
- `cargo test test_grouped_v2_exact_traversal_rejects_invalid_limit_env`

## Measurements

All runtime measurements below were taken on verified grouped-v2 scratch
indexes after confirming:

- `emitted_result_count = 40`
- `grouped_result_count = 40`
- `compared_result_count = 40`

### `layer0` exact traversal only

Scratch runtime was restarted with:

- `TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD=1`
- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN=1`
- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_WINDOW=16`
- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL=1`
- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_SCOPE=layer0`

Representative direct-harness check at `ef_search = 128`:

| corpus | Recall@10 | exact-quantized Recall@10 | mean abs score error | mean query latency ms |
|--------|-----------|---------------------------|----------------------|-----------------------|
| grouped `50k` layer0 exact | 0.8680 | 0.8600 | 0.005645467 | 53.940666 |
| grouped `10k` layer0 exact | 0.9065 | 0.9025 | 0.008688391 | 36.904392 |

Compared to packet `355` all-layer exact traversal:

- `50k`: `0.8780 -> 0.8680`
- `10k`: `0.9380 -> 0.9065`

That was too much quality loss to keep `layer0`-only exact traversal as the
main selective-exact candidate.

### all-layer budgeted exact traversal

The new scratch wrapper:

`./scripts/restart_adr030_scratch.sh --exact-scope all --exact-limit 1`

restarted the same verified scratch lane with:

- `window = 16`
- all-layer exact traversal enabled
- per-expansion exact budget = `1`

Representative external-summary checks at `ef_search = 128`:

| corpus | Recall@10 | Recall@100 | NDCG@10 | mean abs score error | Spearman@10 | exact-quantized Recall@10 | graph_below_exact_queries | worst_exact_gap |
|--------|-----------|------------|---------|----------------------|-------------|---------------------------|---------------------------|-----------------|
| grouped `50k` budget=`1` | 0.8780 | 0.8646 | 0.9198 | 0.005655416 | 0.6768 | 0.8600 | 18 | 2 |
| grouped `10k` budget=`1` | 0.9380 | 0.9363 | 0.9607 | 0.008725143 | 0.8667 | 0.9330 | 0 | 0 |

Those match packet `355`'s all-layer exact traversal summaries exactly at this
operating point.

Direct-harness check at the same `ef_search = 128`:

| corpus | Recall@10 | exact-quantized Recall@10 | mean abs score error | mean query latency ms |
|--------|-----------|---------------------------|----------------------|-----------------------|
| grouped `50k` budget=`1` | 0.8780 | 0.8600 | 0.005655416 | 42.614197 |
| grouped `10k` budget=`1` | 0.9380 | 0.9330 | 0.008725143 | 36.367672 |

For comparison, packet `355` all-layer exact traversal at the same point was:

- `50k`: `44.009502ms`
- `10k`: `38.527134ms`

I also spot-checked `budget = 4` before settling on `1`:

- `50k` and `10k` summary outputs were still identical to all-layer exact
- the budget was therefore wider than necessary for the representative point

## Outcome

This packet sharpens the ADR-030 runtime diagnosis again.

### What it proves

1. exact rescue does **not** need every grouped candidate exact-scored per
   expansion
2. on the verified `10k` and `50k` lanes at `ef=128`, exact-scoring only the
   single best grouped candidate per expansion is enough to preserve packet
   `355`'s full exact-traversal recall summary
3. the scratch wrapper makes the ADR-030 runtime env surface explicit and
   repeatable for future measurement packets

### What it does not prove

1. this is **not** yet a viable grouped-v2 operating point
2. the latency savings from budget `1` over all-layer exact traversal are only
   modest:
   - about `1.40ms` on `50k`
   - about `2.16ms` on `10k`
3. budgeting exact rescoring helps much less than recall alone would suggest,
   which means the branch's remaining cost problem is not "too many exact
   candidates per expansion" by itself

## Next Slice

The next runtime batch should stay on the verified grouped lane, but it should
stop assuming per-expansion exact budget is the dominant cost lever.

Highest-signal follow-ons now look like:

1. measure how many expansions / exact rescoring calls the budget=`1` mode is
   still paying, since that now appears to dominate the remaining cost
2. test an even narrower exact-like seam such as exact rescoring only for the
   frontier admission/head candidate rather than during broader successor
   collection
3. if that still does not collapse the cost, treat "exact-like traversal"
   mainly as a diagnostic tool and pivot back toward improving grouped
   approximate traversal fidelity directly
