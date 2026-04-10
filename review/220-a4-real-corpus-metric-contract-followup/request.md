# Review Request: A4 Real-Corpus Metric Contract Follow-Up

## Context

Branch:
- `fix/a4-real-corpus-recall-lane`

Prior packets:
- `review/218-a4-real-corpus-recall-lane/request.md`
- `review/219-a4-real-corpus-loader-smoke/request.md`

Feedback being addressed here:
- `review/218-a4-real-corpus-recall-lane/feedback/2026-04-09-01-reviewer.md`
- `review/219-a4-real-corpus-loader-smoke/feedback/2026-04-10-01-reviewer.md`

This slice is intentionally narrow:
- fix the real issues in the loader / metric contract surfaced by 219
- remove the avoidable repeated work in the external gate path
- do **not** claim the real-corpus lane is fully complete yet

## What Changed

### 1. External MAE now matches `NFR-003`

The old external summary path computed `mean_abs_score_error` as a rank-profile
L1 distance:
- take the true top-10 fp32 scores
- take the predicted items' fp32 scores
- compare by rank position

That was not `NFR-003`'s metric. The requirement says:
- `mean absolute score error versus true fp32 inner product`

The new external summary path now computes MAE item-by-item for the actual
predicted graph results:
- take the graph-first emitted top-10 items
- read their operator-facing `<#>` scores from
  `debug_gettuple_scan_heap_tids_with_scores(...)`
- negate those back into approximate inner-product space
- compare each predicted item's approximate score to the true fp32 inner
  product for that same item
- average the absolute errors

That is the intended NFR metric.

Files:
- `src/lib.rs`

### 2. The external gate no longer reloads the corpus / queries / fp32 truth four times

Added a shared `ExternalRecallContext` in `src/lib.rs` that builds once:
- loaded corpus ids
- loaded fp32 corpus vectors
- loaded fp32 query vectors
- brute-force fp32 ground truth
- `ctid -> row_index` map
- exact-quantized top-10 row indices per query

`run_graph_scan_recall_gate_from_external(...)` now builds that context once and
reuses it across the four A4 gate configurations instead of rebuilding the same
state per `(m, ef_search)`.

This also removes the old per-query `id_to_row_index_lookup` rebuild.

Files:
- `src/lib.rs`

### 3. Loader now surfaces normalization drift explicitly

`scripts/load_real_corpus.py` now logs L2 norm stats while loading:
- count
- mean norm
- min norm
- max norm

If the vectors are not close to unit norm, it emits a warning. This does not
normalize or reject rows yet; it just makes the metric-contract mismatch visible
at load time instead of silently producing confusing benchmark output.

Files:
- `scripts/load_real_corpus.py`

### 4. Scratch helper portability + docs cleanup

Updated both scratch helpers to derive the default `psql` path from
`${PGRX_HOME:-$HOME/.pgrx}` instead of hardcoding `/home/peter/...`.

Also promoted the stale-extension finding from review 219 into
`docs/RECALL_REAL_CORPUS.md` and clarified why the scratch helper defaults
`PGDATABASE=postgres` while the general docs show `tqvector_bench`.

Files:
- `scripts/load_real_corpus_scratch.sh`
- `scripts/pg17_scratch_psql.sh`
- `docs/RECALL_REAL_CORPUS.md`

## Evidence

### Targeted pg-test still passes

Command:

```bash
cargo test --no-default-features --features 'pg17 pg_test' \
  test_tqhnsw_graph_scan_recall_external_smoke_500 -- --ignored --nocapture
```

Result:
- passes

### Scratch loader now warns immediately on non-unit inputs

Tiny deterministic smoke load on the scratch cluster:

```bash
./scripts/load_real_corpus_scratch.sh \
  --prefix tqhnsw_loader_smoke_v4 \
  --corpus-file /tmp/tqhnsw_loader_smoke_corpus.tsv \
  --queries-file /tmp/tqhnsw_loader_smoke_queries.tsv \
  --m 8 16
```

Observed loader output:

```text
[loader] tqhnsw_loader_smoke_v4_corpus corpus norms: count=24 mean=39.455128 min=35.302161 max=44.131060
[loader] warning: tqhnsw_loader_smoke_v4_corpus corpus vectors do not appear unit-normalized; inner-product/cosine benchmark assumptions may not hold
[loader] tqhnsw_loader_smoke_v4_queries queries norms: count=6 mean=39.965717 min=37.887752 max=44.131046
[loader] warning: tqhnsw_loader_smoke_v4_queries queries vectors do not appear unit-normalized; inner-product/cosine benchmark assumptions may not hold
```

This is exactly the mismatch the 219 reviewer called out. It is now visible
instead of silent.

### Refreshed scratch summary is now interpretable

After reinstalling the current `pg_test` extension into the scratch cluster and
dropping/recreating the extension in the scratch DB, the tiny smoke summary is:

```sql
SELECT corpus_rows, query_count, graph_recall_at_10,
       exact_quantized_recall_at_10, ndcg_at_10, mean_abs_score_error
FROM tests.tqhnsw_graph_scan_recall_external_summary(
  'tqhnsw_loader_smoke_v4_corpus',
  'tqhnsw_loader_smoke_v4_queries',
  'tqhnsw_loader_smoke_v4_m8_idx',
  8,
  128
);
```

Result:
- `24 | 6 | 0.93333334 | 0.93333334 | 0.99119234 | 613.7346`

Interpretation:
- recall / ndcg are still fine for the toy smoke
- MAE remains huge because the staged vectors are wildly non-unit-normalized
  (`mean norm ~= 39-40`)
- that number is now coherent with the load-time warning and the corrected
  item-wise MAE semantics; it is no longer silently masquerading as a
  unit-vector cosine benchmark

## What This Resolves From Review Feedback

From `218`:
- item 2 (`mean_abs_score_error` semantics): addressed
- item 3 (per-query `id_to_row_index_lookup` rebuild): addressed
- item 4 (reload corpus / truth four times): addressed for the heavy state
- item 8 (dead alias): effectively addressed by removing the redundant
  row-index aliasing path in the refactor

From `219`:
- promote stale-extension setup finding into docs: addressed
- add normalization visibility on ingest: addressed
- scratch helper portability: addressed
- scratch DB name inconsistency: addressed in docs

## What Remains Open

### Still blocking for the first official DBpedia number

`218` item 1 remains open:
- canonical subset selection rule
- deterministic parquet -> TSV converter
- manifest hash / reproducibility contract

I do **not** think the first official DBpedia gate should be recorded until that
lands.

### Still follow-on work, but not the blocker for this slice

- `218` item 5: NDCG convention cleanup
- `218` item 7: ann-benchmarks reference anchor
- `218` item 9: the ignored pg-test smoke still does per-row SPI inserts
- `218` item 10: unrelated local dirt in `src/am/options.rs`,
  `tests/recall_integration.rs`, and untracked `spec/adr/ADR-025-...`

## Suggested Next Step

Use this branch state as the new loader baseline, then take the next blocking
real-corpus slice as:
- canonical DBpedia subset rule
- deterministic parquet -> TSV converter
- manifest generation / verification

That can be a separate lane if needed. The current branch now has the plumbing,
the metric semantics, and the repeated-work cleanup in place.
