# Task: Recall Investigation Harness on ExternalRecallContext

Motivation: The A4 real-corpus lane (reviews 218/220/221/222) has just landed
the `ExternalRecallContext` refactor on `fix/a4-real-corpus-recall-lane` and is
about to run the first official DBpedia number against the `ec_hnsw_real_10k`
and `ec_hnsw_real_50k` fixtures. If that first number lands below the 0.89 A4
gate target, coder-1 will immediately need per-query diagnostics to decide
whether the issue is the graph, the quantizer, or `ef_search`. Right now those
diagnostics do not exist. Pre-staging them is the single highest-leverage
parallel task on this lane — it turns the first DBpedia run from a yes/no
result into a diagnosable result.
Priority: batch 1
Status: ready

## Prompt

Add three SQL-callable diagnostic surfaces on top of the already-landed
`ExternalRecallContext` in `src/lib.rs`. These should live in the same
`#[cfg(any(test, feature = "pg_test"))]` block as
`ec_hnsw_graph_scan_recall_external_summary` and
`ec_hnsw_graph_scan_recall_external_gate_report` so they share one install
path.

`ExternalRecallContext` is defined around `src/lib.rs:6306`. It already has:

- `corpus_ids: Vec<i64>`
- `corpus: Vec<Vec<f32>>`
- `queries: Vec<Vec<f32>>`
- `ground_truth_top_k: Vec<Vec<(usize, f32)>>`
- `exact_quantized_row_indices_top10: Vec<Vec<i64>>`
- `ctid_to_row_index: HashMap<(u32, u16), usize>`

All three diagnostics should build the context exactly once per call (via
`build_external_recall_context`) and reuse it — the whole point of the
refactor was that per-config probes are cheap; per-query diagnostics should
pay the same one-time setup cost.

### Surface 1 — per-query recall histogram

`tests.ec_hnsw_graph_scan_recall_histogram(
    corpus_table text,
    query_table text,
    index_name text,
    m integer,
    ef_search integer
)`

Returns a table with columns:

- `recall_bucket integer`  — integer in `[0, 10]`, i.e. number of correct
  top-10 items out of 10
- `query_count integer`    — how many queries landed in this bucket
- `query_fraction float`   — `query_count / total_queries`

Implementation: for each query, run the graph scan once (same path as the
existing summary probe), compute
`|predicted_top_10_row_indices ∩ ground_truth_top_10_row_indices|`, and
bucket by that count. Return one row per bucket in ascending order. Buckets
with zero queries should still be emitted (so the output is always 11 rows).

A healthy 50k/1k run on the real DBpedia fixture should show almost all
queries in buckets 9 and 10. A broken run will spread across buckets. This
histogram is the cheapest way to tell "recall is 0.85 because a few queries
are terrible" apart from "recall is 0.85 because every query is mediocre".

### Surface 2 — `ef_search` sweep on a single fixture

`tests.ec_hnsw_graph_scan_recall_ef_sweep(
    corpus_table text,
    query_table text,
    index_name text,
    m integer,
    ef_values integer[]
)`

Returns a table with columns:

- `m integer`
- `ef_search integer`
- `recall_at_10 float`
- `exact_quantized_recall_at_10 float`
- `mean_abs_score_error float`
- `mean_query_latency_ms float`

One row per `ef_search` value in the input array. Build the context once,
then for each `ef_search` call
`probe_graph_scan_recall_external_summary_for_context` and collect the
summary. This is the fastest way to explore the recall/latency Pareto frontier
on a fixed fixture without rerunning the four hardcoded `RECALL_GATE_CONFIGS`
rows from `run_graph_scan_recall_gate_from_external`. Suggested default
parameter grid when invoking: `ARRAY[40, 64, 100, 128, 160, 200, 300, 500]`.

### Surface 3 — exact-vs-approximate diff for failing queries

`tests.ec_hnsw_graph_scan_recall_failure_breakdown(
    corpus_table text,
    query_table text,
    index_name text,
    m integer,
    ef_search integer,
    recall_threshold integer
)`

Returns one row per query whose top-10 recall count is **strictly less than**
`recall_threshold` (e.g. pass `8` to list every query that missed at least 3
of 10). Columns:

- `query_index integer`      — position of the query in `queries`
- `graph_recall_at_10 integer`          — how many correct items the graph got
- `exact_quantized_recall_at_10 integer` — how many correct items the
  exact-quantized top-10 got for the same query (from
  `ExternalRecallContext::exact_quantized_row_indices_top10`)
- `missed_ids bigint[]`      — ground truth top-10 ids that neither the graph
  nor exact-quantized top-10 found

This is the single diagnostic that distinguishes "the graph is the bottleneck"
from "the quantizer is the bottleneck". If a query misses 4/10 for the graph
but 0/10 for exact-quantized, the graph is the problem. If both miss the same
items, the quantizer is the problem and no amount of `ef_search` will fix it.

## Design notes

- Do **not** duplicate the context build or the brute-force ground truth
  computation. Reuse `build_external_recall_context` exactly as the existing
  gate report does. Each new surface should look like a thin wrapper:
  `build context; do thing; return table.`
- All three surfaces go in the same `mod tests` block that contains the
  existing `ec_hnsw_graph_scan_recall_external_*` surfaces. Gate them behind
  `#[cfg(any(test, feature = "pg_test"))]`. Do not expose them as
  first-class extension functions.
- Use `pgrx::pg_extern` with `SETOF` return types. Follow the pattern in
  `ec_hnsw_graph_scan_recall_external_gate_report` for the composite-row
  return shape.
- Do not add new corpus-loading paths. The existing
  `build_external_recall_context` already loads from the canonical
  `<prefix>_corpus` and `<prefix>_queries` tables.
- The failure-breakdown surface should emit rows in `query_index` order so the
  output is deterministic for diffing.
- Document each new surface in `docs/RECALL_REAL_CORPUS.md` under a new
  `## Diagnostics` section, with one worked example per surface. Keep the
  examples short and concrete (e.g. `SELECT * FROM
  ec_hnsw_graph_scan_recall_histogram('ec_hnsw_real_10k_corpus', ...)` with
  expected output shape).

## Out of scope

- Latency benchmarking (covered by task 10057).
- Binary NDCG convention fix — coder-1 owns that on the primary lane.
- Altering `ExternalRecallContext` itself. It is considered stable; if you
  need a field it does not expose, add the derivation inside your new
  surface rather than widening the context struct.

## Validate

```bash
cargo clippy --all-targets --no-default-features --features 'pg17 pg_test' -- -D warnings
cargo pgrx test pg17
```

Manually smoke against the scratch cluster once the real `ec_hnsw_real_10k`
fixture is loaded (the loader is in `scripts/load_real_corpus_scratch.sh`):

```sql
SELECT * FROM tests.ec_hnsw_graph_scan_recall_histogram(
    'ec_hnsw_real_10k_corpus', 'ec_hnsw_real_10k_queries',
    'ec_hnsw_real_10k_m8_idx', 8, 128);

SELECT * FROM tests.ec_hnsw_graph_scan_recall_ef_sweep(
    'ec_hnsw_real_10k_corpus', 'ec_hnsw_real_10k_queries',
    'ec_hnsw_real_10k_m8_idx', 8, ARRAY[40, 64, 100, 128, 200]);

SELECT * FROM tests.ec_hnsw_graph_scan_recall_failure_breakdown(
    'ec_hnsw_real_10k_corpus', 'ec_hnsw_real_10k_queries',
    'ec_hnsw_real_10k_m8_idx', 8, 128, 8);
```

Record the observed outputs in the review packet so coder-1 can use them
against the 50k fixture without rediscovering the invocation.

Branch from current upstream main. Push branch for review.
