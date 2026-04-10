# Review Request: A4 Recall Investigation Harness

## Context

Branch:
- `feat/10054-recall-investigation-harness`

Task spec:
- `plan/tasks/coder2/10054-recall-investigation-harness.md`

Prior packets in the A4 real-corpus lane:
- `review/218-a4-real-corpus-recall-lane/request.md`
- `review/219-a4-real-corpus-loader-smoke/request.md`
- `review/220-a4-real-corpus-metric-contract-followup/request.md`
- `review/221-a4-real-corpus-subset-manifest-contract/request.md`
- `review/222-a4-real-corpus-fetch-and-schema-alignment/request.md`

The A4 real-corpus lane has just landed `ExternalRecallContext` and is about to
record the first official DBpedia gate number against the canonical
`tqhnsw_real_10k` and `tqhnsw_real_50k` fixtures. If that first number lands
below the `0.89` A4 gate target, coder-1 will immediately need per-query
diagnostics to decide whether the issue is the graph, the quantizer, or
`ef_search`. Before this slice those diagnostics did not exist — every
sub-gate result would have been a yes/no with no path forward.

This packet pre-stages those diagnostics so the first DBpedia run is
diagnosable instead of binary.

## What Landed

### 1. Per-query top-10 recall histogram

New pg_test surface:

```
tqhnsw_graph_scan_recall_histogram(
    corpus_table text,
    query_table  text,
    index_name   text,
    m            integer,
    ef_search    integer
) RETURNS TABLE (
    recall_bucket  integer,  -- 0..=10
    query_count    integer,
    query_fraction float
)
```

Always emits 11 rows. Bins each query's top-10 graph recall against the
precomputed fp32 ground truth that the existing `ExternalRecallContext`
already exposes. The shape of the histogram is the cheapest way to tell
"a few queries are catastrophic, the rest are fine" apart from "every query
is mediocre" — the two failure modes have completely different fixes.

### 2. `ef_search` sweep on a single fixture

New pg_test surface:

```
tqhnsw_graph_scan_recall_ef_sweep(
    corpus_table text,
    query_table  text,
    index_name   text,
    m            integer,
    ef_values    integer[]
) RETURNS TABLE (
    m                            integer,
    ef_search                    integer,
    recall_at_10                 float,
    exact_quantized_recall_at_10 float,
    mean_abs_score_error         float,
    mean_query_latency_ms        float
)
```

Builds the recall context exactly once and reuses it across every
`ef_search` value. This is the fastest way to walk the recall/latency
Pareto frontier on a fixed fixture without rerunning the four hardcoded
`RECALL_GATE_CONFIGS` rows. `mean_query_latency_ms` is the wall-clock spent
inside the existing summary probe divided by query count — it includes the
small per-query NDCG/MAE/Spearman bookkeeping cost but is dominated by the
graph traversal itself, which is the surface of interest for sweeping.

### 3. Exact-vs-approximate diff for failing queries

New pg_test surface:

```
tqhnsw_graph_scan_recall_failure_breakdown(
    corpus_table     text,
    query_table      text,
    index_name       text,
    m                integer,
    ef_search        integer,
    recall_threshold integer
) RETURNS TABLE (
    query_index                  integer,
    graph_recall_at_10           integer,
    exact_quantized_recall_at_10 integer,
    missed_ids                   bigint[]
)
```

Lists every query whose graph top-10 recall is **strictly less than**
`recall_threshold`, in `query_index` order so the output is deterministic
for diffing. `missed_ids` is the set of corpus ids that neither the graph
top-10 nor the precomputed exact-quantized top-10 found — i.e. ids that
*both* retrieval surfaces missed.

This is the single diagnostic that distinguishes "the graph is the
bottleneck" from "the quantizer is the bottleneck". If a query misses 4/10
on the graph but 0/10 on exact-quantized, raising `ef_search` will help. If
both surfaces miss the same items, no amount of `ef_search` tuning will
help; the quantizer is the floor.

### 4. Documentation

`docs/RECALL_REAL_CORPUS.md` gained a new `## Diagnostics` section that
slots in between the existing per-query summary and the Reporting section.
The new section gives one worked example per surface with the expected
output shape, so the harness is reachable from the same doc that already
walks reviewers through the load and gate-report path.

## Design notes followed

- The new surfaces live in the same `#[cfg(any(test, feature = "pg_test"))]`
  block as the existing `tqhnsw_graph_scan_recall_external_summary` /
  `tqhnsw_graph_scan_recall_external_gate_report`, so they install via the
  same path and gate behind the same `pg_test` build.
- Each surface is a thin wrapper around `build_external_recall_context`
  followed by a small per-context routine. The recall context is built
  exactly once per call, mirroring the gate report.
- `ExternalRecallContext` itself was not modified. The failure breakdown
  derives the missed ids from the existing
  `exact_quantized_row_indices_top10` and `corpus_ids` fields rather than
  widening the struct.
- The histogram and breakdown use `am::debug_gettuple_scan_heap_tids`
  (no scores) since neither needs MAE; the sweep delegates to
  `probe_graph_scan_recall_external_summary_for_context` so it inherits the
  exact same numerics as the existing summary surface.
- Failure-breakdown rows are emitted in `query_index` order for
  deterministic diffing.
- The pg_extern signatures take `m` for parity with the other recall
  diagnostics even though `m` is fully encoded in `index_name` for the
  histogram and breakdown — this keeps the call shape consistent across
  the four surfaces a reviewer might invoke from `psql`.

## Evidence

### Lint clean

```
cargo clippy --all-targets --no-default-features --features 'pg17 pg_test' \
    -- -D warnings
```

```
    Checking tqvector v0.1.0 (/home/peter/dev/tqvector)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.88s
```

(exit 0)

### pg_test green for the recall lane

```
cargo pgrx test pg17 --no-default-features --features 'pg17 pg_test' recall
```

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.62s
     Writing SQL entities to /home/peter/.pgrx/17.9/pgrx-install/share/postgresql/extension/tqvector--0.1.0.sql
    Finished installing tqvector
test tests::pg_test_tqhnsw_graph_scan_recall_gate ... ok

test result: ok. 1 passed; 0 failed; 7 ignored; 0 measured; 240 filtered out; finished in 17.09s
...
test result: ok. 2 passed; 0 failed; 20 ignored; 0 measured; 0 filtered out; finished in 0.12s
```

The new surfaces are pgrx `#[pg_extern]` functions inside the same
`#[pg_test]` mod as the existing external surfaces, so SQL entities for
them are emitted by the same install pass that the recall gate test
exercises. The existing `pg_test_tqhnsw_graph_scan_recall_gate` continues
to pass against the synthetic fixture.

### Manual smoke against the scratch cluster

The validation block in `plan/tasks/coder2/10054-recall-investigation-harness.md`
calls for a manual smoke against the scratch cluster once the real
`tqhnsw_real_10k` fixture is loaded. That smoke is **not** included in this
packet — coder-1 owns the real fixture loader run and the first official
DBpedia gate-report number; the new diagnostics are pre-staged so coder-1
can pivot into them the moment the gate report comes back below target.
The intended invocations on the scratch cluster, copy-pasteable from
`docs/RECALL_REAL_CORPUS.md`, are:

```sql
SELECT * FROM tqhnsw_graph_scan_recall_histogram(
    'tqhnsw_real_10k_corpus',
    'tqhnsw_real_10k_queries',
    'tqhnsw_real_10k_m8_idx',
    8,
    128
);

SELECT * FROM tqhnsw_graph_scan_recall_ef_sweep(
    'tqhnsw_real_10k_corpus',
    'tqhnsw_real_10k_queries',
    'tqhnsw_real_10k_m8_idx',
    8,
    ARRAY[40, 64, 100, 128, 160, 200, 300, 500]
);

SELECT * FROM tqhnsw_graph_scan_recall_failure_breakdown(
    'tqhnsw_real_10k_corpus',
    'tqhnsw_real_10k_queries',
    'tqhnsw_real_10k_m8_idx',
    8,
    128,
    8
);
```

When the first DBpedia number lands, capture the output of each of the
three calls into the corresponding gate-report review packet so the
diagnosis is preserved alongside the number it diagnoses.

## Why This Matters

A4's primary risk right now is "the first real DBpedia number lands below
gate, and we have no diagnostic surface to decide what to fix". This packet
removes that risk before the first number is recorded:

- the histogram tells us *how many* queries are bad, not just *whether*
  recall is bad
- the sweep tells us whether `ef_search` alone can close the gap on the
  same fixture
- the breakdown tells us, per failing query, whether the floor is the
  graph or the quantizer

Together they turn the first DBpedia run from a yes/no result into a
diagnosable result, which is exactly the precondition the gate-report
review packet will need.

## Out of Scope

- Latency benchmarking (covered by task `10056`).
- The binary NDCG convention fix (owned by coder-1 on the primary lane).
- Any changes to `ExternalRecallContext` itself.
- Loading the real DBpedia fixture or running the manual smoke against
  it — coder-1 owns that run as part of recording the first official
  gate number.

## Files

- `src/lib.rs`
  - new type aliases for the three return shapes
  - new helpers: `build_graph_scan_recall_histogram_for_context`,
    `run_graph_scan_recall_ef_sweep_for_context`,
    `run_graph_scan_recall_failure_breakdown_for_context`
  - new pg_extern surfaces: `tqhnsw_graph_scan_recall_histogram`,
    `tqhnsw_graph_scan_recall_ef_sweep`,
    `tqhnsw_graph_scan_recall_failure_breakdown`
- `docs/RECALL_REAL_CORPUS.md`
  - new `## Diagnostics` section with one worked example per surface
- `review/223-a4-recall-investigation-harness/request.md` (this file)
