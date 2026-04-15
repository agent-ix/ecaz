# Review Request: C1 ADR-030 V2 Grouped Exact Cost Profile

## Context

Packet `354` corrected the grouped-v2 runtime story on verified `10k` and `50k`
indexes:

- grouped approximate traversal is much faster than scalar
- grouped approximate traversal also loses far too much recall to be a usable
  general path
- all-layer exact traversal on the same grouped-built graph recovers recall close
  to scalar, which means the graph is not the main failure

Packet `356` then explored per-expansion exact budgets and concluded that
`budget = 1` preserved the all-layer exact recall summary while only shaving a
small amount of cost.

Reviewer feedback on `354` and `356` pushed two follow-ons before making a
larger runtime claim:

1. add a structural settings audit so grouped runtime measurements stop relying
   on implicit scratch env state
2. measure the exact-like runtime with real hot-path counters instead of
   inferring cost only from end-to-end latency

## Problem

The branch still lacked a verified answer to two narrow questions:

1. where the grouped exact-like runtime is actually spending time on the verified
   `50k` lane
2. whether the per-expansion exact budget family really has a viable quality/cost
   operating point once measured on the same verified lane with the new hot-path
   counters

The scratch workflow also still needed an explicit, repeatable way to refresh
test-only debug wrappers after code changes without ad hoc SQL edits.

## Planned Slice

Batch the next tightly related slices together:

1. add grouped traversal hot-path counters for approximate scoring, exact
   scoring, and budgeted-exact expansion counts
2. route grouped exact traversal and grouped emitted comparison scoring through
   the shared per-scan score cache so repeated exact rescoring shows up as cache
   reuse instead of repeated cold work
3. expose a backend-visible ADR-030 runtime settings probe in the `tests` schema
4. add a repo-local scratch helper to refresh the debug SQL wrappers against the
   already-loaded module path
5. remeasure the verified `50k` grouped lane at `ef_search = 128` on:
   - approximate traversal
   - all-layer exact traversal
   - all-layer exact budget `1`
   - all-layer exact budget `4`
   - all-layer exact budget `8`

This slice intentionally does not:

- lift any ADR-030 gate
- change the grouped-v2 on-disk format
- claim a planner-facing operating point

## Implementation

Updated:

- `src/am/scan.rs`
- `src/am/scan_debug.rs`
- `src/lib.rs`
- `scripts/refresh_adr030_scratch_debug_helpers.sh`
- `scripts/sql/refresh_adr030_scratch_debug_helpers.sql`

Concrete changes:

1. extended `ScanDebugProfile` with grouped traversal counters for:
   - grouped approximate score calls / elapsed time
   - grouped exact score calls / elapsed time
   - budgeted exact expansions / candidates / exact candidates
2. timed grouped approximate traversal scoring explicitly and surfaced that work
   through the existing debug profile helper
3. added `exact_score_grouped_candidate_context(...)` so grouped exact traversal
   and grouped emitted comparison scoring reuse the shared scan-local score cache
   keyed by graph element TID
4. changed grouped exact traversal profiling to measure the whole exact miss path
   around cache lookup / cold payload load / exact score production, instead of
   only the quantizer call
5. kept grouped traversal-specific timing separate from the generic
   `candidate_score_elapsed` counter so the new counters explain grouped exact
   work without double-counting the generic score timer
6. added `tests.tqhnsw_debug_adr030_runtime_settings()` so scratch runs can
   verify the backend-visible ADR-030 gate surface directly:
   - grouped build gate
   - grouped scan gate
   - grouped scan window
   - exact traversal gate
   - exact traversal scope
   - exact traversal limit
7. added pg coverage for:
   - the new runtime settings probe
   - grouped approximate profiles leaving grouped exact counters inert
   - budgeted exact grouped traversal surfacing both exact work and score-cache
     reuse
8. added a repo-local scratch helper and SQL file to recreate the debug wrapper
   signatures in an already-loaded scratch DB by reusing the live module path
   from an existing `tests.*` C wrapper

## Validation

Required checkpoint validation passed on the final code:

- `cargo test`
- `/bin/bash -lc 'PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17'`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Focused checks also passed while landing the slice:

- `cargo test test_tqhnsw_debug_adr030_runtime_settings_reflect_env -- --nocapture`
- `cargo test grouped_v2_runtime_profile_budgeted_exact_counters -- --nocapture`

## Measurements

All runtime measurements below were taken on the verified grouped-v2
`tqhnsw_real_50k_grouped_m8_idx` scratch lane after confirming:

- `emitted_result_count = 40`
- `grouped_result_count = 40`
- `compared_result_count = 40`

The hot-path profile numbers below are averages over the first `10` queries from
`tqhnsw_real_50k_queries_50`. The recall summary numbers are the full
`50`-query external summary at `ef_search = 128`.

### approximate traversal (`window = 16`)

Backend settings probe:

- build=`true`
- scan=`true`
- window=`16`
- exact traversal=`false`

Hot-path sample:

| amrescan us | graph element load us | grouped approx us | grouped exact us | candidate score calls | score cache hits | score cache misses |
|-------------|------------------------|-------------------|------------------|-----------------------|------------------|--------------------|
| 14499.6 | 4596.6 | 650.7 | 0.0 | 625.8 | 0.0 | 16.0 |

External summary:

| Recall@10 | Recall@100 | NDCG@10 | mean abs score error | Spearman@10 | exact-quantized Recall@10 | graph_below_exact_queries | worst_exact_gap |
|-----------|------------|---------|----------------------|-------------|---------------------------|---------------------------|-----------------|
| 0.6740 | 0.5572 | 0.7754 | 0.24845067 | 0.1706667 | 0.8600 | 39 | 5 |

### all-layer exact traversal (`window = 16`)

Backend settings probe:

- build=`true`
- scan=`true`
- window=`16`
- exact traversal=`true`
- scope=`all`
- limit=`NULL`

Hot-path sample:

| amrescan us | graph element load us | grouped approx us | grouped exact us | candidate score calls | score cache hits | score cache misses |
|-------------|------------------------|-------------------|------------------|-----------------------|------------------|--------------------|
| 20951.6 | 3912.2 | 0.0 | 8139.8 | 539.4 | 207.2 | 539.4 |

External summary:

| Recall@10 | Recall@100 | NDCG@10 | mean abs score error | Spearman@10 | exact-quantized Recall@10 | graph_below_exact_queries | worst_exact_gap |
|-----------|------------|---------|----------------------|-------------|---------------------------|---------------------------|-----------------|
| 0.8780 | 0.8646 | 0.9198 | 0.005655416 | 0.67684835 | 0.8600 | 18 | 2 |

### all-layer exact budget `1`

Backend settings probe:

- build=`true`
- scan=`true`
- window=`16`
- exact traversal=`true`
- scope=`all`
- limit=`1`

Hot-path sample:

| amrescan us | graph element load us | grouped approx us | grouped exact us | grouped exact calls | budgeted exact candidates | score cache hits | score cache misses |
|-------------|------------------------|-------------------|------------------|---------------------|---------------------------|------------------|--------------------|
| 16030.4 | 4657.5 | 747.5 | 950.3 | 53.7 | 52.7 | 65.8 | 53.7 |

External summary:

| Recall@10 | Recall@100 | NDCG@10 | mean abs score error | Spearman@10 | exact-quantized Recall@10 | graph_below_exact_queries | worst_exact_gap |
|-----------|------------|---------|----------------------|-------------|---------------------------|---------------------------|-----------------|
| 0.4900 | 0.5048 | 0.6439 | 0.0055784225 | -0.17272724 | 0.8600 | 49 | 7 |

This does **not** reproduce packet `356`'s earlier `budget = 1` parity claim on
the verified `50k` lane. Treat that earlier measurement claim as superseded by
this packet.

### all-layer exact budget `4`

Backend settings probe:

- build=`true`
- scan=`true`
- window=`16`
- exact traversal=`true`
- scope=`all`
- limit=`4`

Hot-path sample:

| amrescan us | graph element load us | grouped approx us | grouped exact us | grouped exact calls | budgeted exact candidates | score cache hits | score cache misses |
|-------------|------------------------|-------------------|------------------|---------------------|---------------------------|------------------|--------------------|
| 19856.4 | 4873.3 | 759.7 | 3784.9 | 215.9 | 214.9 | 146.3 | 215.9 |

External summary:

| Recall@10 | Recall@100 | NDCG@10 | mean abs score error | Spearman@10 | exact-quantized Recall@10 | graph_below_exact_queries | worst_exact_gap |
|-----------|------------|---------|----------------------|-------------|---------------------------|---------------------------|-----------------|
| 0.8460 | 0.6734 | 0.8993 | 0.0056869616 | 0.6330909 | 0.8600 | 21 | 3 |

### all-layer exact budget `8`

Backend settings probe:

- build=`true`
- scan=`true`
- window=`16`
- exact traversal=`true`
- scope=`all`
- limit=`8`

Hot-path sample:

| amrescan us | graph element load us | grouped approx us | grouped exact us | grouped exact calls | budgeted exact candidates | score cache hits | score cache misses |
|-------------|------------------------|-------------------|------------------|---------------------|---------------------------|------------------|--------------------|
| 21642.0 | 4202.4 | 656.4 | 6498.3 | 394.9 | 393.9 | 187.3 | 394.9 |

External summary:

| Recall@10 | Recall@100 | NDCG@10 | mean abs score error | Spearman@10 | exact-quantized Recall@10 | graph_below_exact_queries | worst_exact_gap |
|-----------|------------|---------|----------------------|-------------|---------------------------|---------------------------|-----------------|
| 0.8800 | 0.8384 | 0.9211 | 0.0056661326 | 0.67672724 | 0.8600 | 18 | 2 |

Within normal run-to-run noise, `budget = 8` is effectively the same quality as
all-layer exact traversal while paying almost the same exact-work bill.

## Outcome

This packet sharpens the grouped exact-like diagnosis substantially.

### What it proves

1. the new hot-path counters and runtime settings probe close the measurement
   hygiene gap called out in reviewer feedback on `354`
2. grouped exact-like runtime cost on verified `50k` is dominated by exact-score
   miss count, not by a small amount of generic scan scaffolding
3. the grouped score cache is actually being reused on exact-like modes once the
   exact path is routed through the shared cache
4. there is a real budget ladder on the verified `50k` lane:
   - `budget = 1` is cheap but destroys quality
   - `budget = 4` is a real middle point
   - `budget = 8` is basically full exact again

### What it does not prove

1. this is still **not** a viable grouped-v2 operating point
2. the budget family no longer supports the earlier claim that `budget = 1`
   preserves all-layer exact quality
3. `budget = 8` does not buy enough cost reduction to justify itself over full
   exact traversal

## Next Slice

The next runtime batch should stop treating "per-expansion exact budget" as the
main path to viability.

Highest-signal follow-ons now look like:

1. treat packet `356`'s `budget = 1` parity claim as superseded and keep this
   packet as the authoritative exact-budget ladder
2. test a narrower exact-like seam that exact-scores only the frontier
   admission/head candidate instead of an arbitrary prefix of every expansion
3. if that still fails to produce a better quality/cost knee than `budget = 4`,
   treat exact-like traversal as a diagnostic / fallback lane and return to
   improving grouped approximate candidate quality directly
