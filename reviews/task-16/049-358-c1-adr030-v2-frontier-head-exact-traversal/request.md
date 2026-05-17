# Review Request: C1 ADR-030 V2 Frontier-Head Exact Traversal

## Context

Packet `357` profiled the verified grouped-v2 `50k` lane and closed the loop on
the per-expansion exact-budget family:

- approximate grouped traversal at `window = 16` was still far below scalar
  (`Recall@10 = 0.674` at `ef_search = 128`)
- full exact traversal on the same grouped-built graph recovered recall close to
  scalar (`0.878`)
- budgeted exact traversal showed that cheap per-expansion rescoring did not
  hold up on the verified lane:
  - `budget = 1` collapsed to `0.490`
  - `budget = 4` improved to `0.846`
  - `budget = 8` was basically full exact again

That left one narrower candidate-quality seam still worth testing before
assuming the grouped approximate scorer itself needed redesign:

- instead of exact-scoring a prefix of every discovered successor set, exactify
  only the current visible frontier head and re-run source selection from the
  updated frontier

The working hypothesis was that this might recover most of the useful traversal
decisions with far fewer exact misses than packet `357`'s budgeted-per-expansion
family.

## Problem

The branch still lacked a verified answer to this specific question:

> Does exactifying only the current layer-0 frontier head materially improve
> grouped-v2 recall on the verified `10k` / `50k` lane at a much lower cost than
> the existing exact-like traversal modes?

The scratch debug-helper refresh workflow also broke once the
`tests.tqhnsw_debug_adr030_runtime_settings()` wrapper gained a new output
column, because PostgreSQL does not allow `CREATE OR REPLACE FUNCTION` to change
the OUT row type in place.

## Planned Slice

Batch the tightly related work together:

1. add an explicit grouped exact-traversal strategy surface with a new
   `frontier_head` mode
2. keep the existing `expansion` behavior as the default strategy, so the prior
   packets keep their meaning unchanged
3. restrict `frontier_head` to `scope = layer0`, since the live visible-frontier
   scheduler only exists on the layer-0 traversal path
4. before consuming the next grouped visible frontier node, exact-score the
   current best frontier head, update its visible score, and reseed the small
   scheduler from the updated visible frontier
5. expose the resolved exact-traversal strategy in the runtime settings probe
   and scratch restart wrapper
6. fix the scratch debug-helper SQL refresh workflow so wrapper signature
   changes can be applied safely in-place
7. measure the verified grouped-v2 `50k` and `10k` lane at
   `ef_search = 128, window = 16` with:
   - `scope = layer0`
   - `strategy = frontier_head`

This slice intentionally does not:

- claim a planner-facing operating point
- change the grouped-v2 on-disk format
- change the grouped approximate scorer itself

## Implementation

Updated:

- `src/am/search.rs`
- `src/am/scan.rs`
- `src/lib.rs`
- `scripts/restart_adr030_scratch.sh`
- `scripts/sql/refresh_adr030_scratch_debug_helpers.sql`

Concrete changes:

1. added `GroupedExactTraversalStrategy` with:
   - `expansion` (existing behavior; default)
   - `frontier_head` (new experiment)
2. resolved the strategy once per `amrescan` from
   `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_STRATEGY`
3. rejected `frontier_head` unless
   `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_SCOPE = layer0`
4. changed grouped exact traversal dispatch so:
   - `expansion + no limit` still does full exact grouped candidate scoring
   - `expansion + limit` still does packet `357`'s budgeted candidate rescoring
   - `frontier_head` leaves grouped candidate generation approximate and moves
     exact scoring to the visible frontier head refinement step
5. added a visible-frontier candidate replacement helper in `src/am/search.rs`
   so the exactified frontier head can replace its approximate score in place
6. added `refine_grouped_frontier_head_exact(...)` in `src/am/scan.rs`:
   - reads the current best visible frontier candidate
   - exact-scores it through the existing grouped exact scoring path
   - updates the visible frontier score in place
   - reseeds the small scheduler from the updated visible frontier
   - repeats until the current visible best already has an exact cached score
7. invoked the new refinement step in both grouped live graph-result paths
   before consuming the next frontier head
8. extended `tests.tqhnsw_debug_adr030_runtime_settings()` to surface the exact
   traversal strategy
9. added pg coverage for:
   - runtime settings exposing the strategy surface
   - invalid strategy env rejection
   - frontier-head runtime profiles surfacing exact traversal work while leaving
     the budgeted-exact counters at zero
10. extended `scripts/restart_adr030_scratch.sh` with `--exact-strategy`
11. fixed `scripts/sql/refresh_adr030_scratch_debug_helpers.sql` by explicitly
    dropping `tests.tqhnsw_debug_adr030_runtime_settings()` before recreating it
    with the new OUT column layout

## Validation

Required checkpoint validation passed on the final code:

- `cargo test`
- `/bin/bash -lc 'PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17'`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Scratch workflow validation also passed:

- `./scripts/restart_adr030_scratch.sh --window 16 --exact-scope layer0 --exact-strategy frontier_head`
- `./scripts/pg17_scratch_psql.sh --file scripts/sql/refresh_adr030_scratch_debug_helpers.sql`
- `./scripts/pg17_scratch_psql.sh --sql "select ... from tests.tqhnsw_debug_adr030_runtime_settings();"`

Verified scratch settings after restart:

- build=`true`
- scan=`true`
- window=`16`
- exact traversal=`true`
- scope=`layer0`
- strategy=`frontier_head`
- limit=`NULL`

Verified scratch grouped lane after restart:

- `tqhnsw_real_50k_grouped_m8_idx` still reports
  `emitted_result_count = 40`, `grouped_result_count = 40`,
  `compared_result_count = 40`

## Measurements

All measurements below were taken on the verified grouped-v2 lane at
`window = 16` with:

- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL=1`
- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_SCOPE=layer0`
- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_STRATEGY=frontier_head`

### Verified `50k` grouped-v2 (`tqhnsw_real_50k_grouped_m8_idx`)

External summary at `ef_search = 128`:

| Recall@10 | Recall@100 | NDCG@10 | mean abs score error | Spearman@10 | exact-quantized Recall@10 | graph_below_exact_queries | worst_exact_gap |
|-----------|------------|---------|----------------------|-------------|---------------------------|---------------------------|-----------------|
| 0.6740 | 0.5572 | 0.7754 | 0.0055493773 | 0.1706667 | 0.8600 | 39 | 5 |

This is effectively unchanged from packet `357`'s approximate grouped runtime
at the same operating point.

Hot-path sample over the first `10` queries:

| amrescan us | graph element load us | grouped approx us | grouped exact us | candidate score calls | score cache hits | score cache misses | grouped approx calls | grouped exact calls | budgeted exact candidates |
|-------------|------------------------|-------------------|------------------|-----------------------|------------------|--------------------|----------------------|---------------------|---------------------------|
| 13381.5 | 2675.0 | 637.9 | 308.9 | 626.0 | 16.0 | 16.2 | 609.8 | 16.2 | 0.0 |

Interpretation:

- the new strategy definitely runs and exact-scores some layer-0 frontier heads
- it is materially cheaper than all-layer exact and budget `4`
- it does **not** improve the emitted candidate set enough to move recall

### Verified `10k` grouped-v2 (`tqhnsw_real_10k_grouped_m8_idx`)

External summary at `ef_search = 128` on the verified grouped corpus/query
tables:

| Recall@10 | Recall@100 | NDCG@10 | mean abs score error | Spearman@10 | exact-quantized Recall@10 | graph_below_exact_queries | worst_exact_gap |
|-----------|------------|---------|----------------------|-------------|---------------------------|---------------------------|-----------------|
| 0.8155 | 0.6714 | 0.8718 | 0.008714174 | 0.6325757 | 0.7965 | 6 | 1 |

This also matches the approximate grouped runtime operating point instead of
showing a meaningful recall lift.

Hot-path sample over the first `10` verified grouped queries:

| amrescan us | graph element load us | grouped approx us | grouped exact us | candidate score calls | score cache hits | score cache misses | grouped approx calls | grouped exact calls |
|-------------|------------------------|-------------------|------------------|-----------------------|------------------|--------------------|----------------------|---------------------|
| 12160.3 | 1787.9 | 658.7 | 338.1 | 593.4 | 16.0 | 16.3 | 577.1 | 16.3 |

## Outcome

This packet closes the frontier-head exactification experiment with a negative
result:

- exactifying only the current visible frontier head is too weak to change the
  grouped-v2 candidate set on the verified `50k` / `10k` lane
- the strategy does real exact traversal work, but it produces essentially the
  same recall summary as plain approximate grouped traversal
- the cheap exact-like seams now tested on the verified lane are:
  - per-expansion budget `1` (bad recall)
  - frontier-head exactification (no measurable recall lift)

That makes the next direction much clearer:

- the viable grouped-v2 path is no longer “find a cheaper exact-like traversal
  seam around the current grouped approximate scorer”
- the next useful work should target the grouped approximate candidate scorer
  itself or a different candidate-selection signal, because the current grouped
  approximate ordering is too lossy upstream of rerank

## Open Questions

1. Is there a better approximate candidate-selection signal already available on
   grouped-v2 pages, such as a binary-sidecar-based scan mode, that is more
   faithful than the current grouped PQ search-code score?
2. If not, does grouped-v2 need a different hot traversal payload entirely for
   candidate selection, with the current grouped search-code score retained only
   as a secondary signal?
3. Should the next runtime investigation stay on scan-time alternatives only, or
   does the grouped build path also need to emit additional candidate-selection
   state to make a viable grouped runtime possible?
