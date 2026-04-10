# Review Request: A4 ann-benchmarks Reference Anchor

## Context

Task: `plan/tasks/coder2/10055-ann-benchmarks-reference-anchor.md`
Branch: `feat/10055-ann-benchmarks-anchor`
Off main: `ef685d7 Add coder-2 parallel tasks for A4 real-corpus lane`

Prior packets in the same A4 lane:
- `review/218-a4-real-corpus-recall-lane/request.md`
- `review/220-a4-real-corpus-metric-contract-followup/request.md`
- `review/221-a4-real-corpus-subset-manifest-contract/request.md`
- `review/222-a4-real-corpus-fetch-and-schema-alignment/request.md`

Review 218 item 7 flagged that the real-corpus A4 lane has no external
oracle: if `build_external_recall_context` or
`scripts/qdrant_dbpedia_to_tsv.py` ever silently corrupts the corpus, every
real-corpus gate run is a self-referential measurement and nobody can
spot-check it. Reviews 220, 221, and 222 explicitly deferred this item until
the canonical loader path was proven on the actual DBpedia parquet.

That precondition is now met (review 222), so this packet lands the anchor.
It is intentionally a one-shot oracle, not a sweep — sweep diagnostics live
in task 10054's surfaces.

## What Landed

### 1. The published anchor row is documented

`docs/RECALL_ANN_BENCHMARKS_ANCHOR.md` records the single row we anchor
against, copied verbatim from the Qdrant `vector-db-benchmark` published
results JSON:

| Field | Value |
| --- | --- |
| Source URL | `https://qdrant.tech/benchmarks/results-1-100-thread-2024-06-15.json` |
| Engine | `qdrant` |
| Setup name | `qdrant-m-16-ef-128` |
| Dataset | `dbpedia-openai-1M-1536-angular` (cosine, 1536-dim) |
| Build `m` | `16` |
| Build `ef_construct` | `128` |
| Search `hnsw_ef` | `128` |
| Published `recall@10` | `0.96082` |

The setup-name → search-parameter mapping is pinned by the matching config
file at
`https://raw.githubusercontent.com/qdrant/vector-db-benchmark/master/experiments/configurations/qdrant-single-node.json`,
under the `qdrant-m-16-ef-128` config: the `parallel=1` lane sweeps
`hnsw_ef ∈ [64, 128, 256, 512]`, and the four `parallel=1` JSON entries
return `recall@10` = `[0.94978, 0.96082, 0.96640, 0.96902]` in that order.
The `hnsw_ef=128` row is the second entry.

The doc explains:

- Why this row and not another (1536-dim, public, source-controlled,
  reuses the existing build path's `m=16`/`ef_construct=128`/`ef_search=128`).
- That cosine and inner-product-on-unit-norm produce the same ranking, so
  `recall@10` is directly comparable, and points to the loader's
  `VectorNormStats` warning as the verification surface.
- That the parquet release we convert from has exactly 1M rows with no
  separate query split, and that the anchor profile therefore reuses the
  same canonical sorted-id rule used by the rest of the lane and treats the
  last 10k rows as the query set (corpus = 990k). The 0.02 tolerance is
  what absorbs that gap.
- That the anchor must use the `build_source_column = 'source'` build path
  (same as the primary gate).
- The exact reproduction recipe: converter command, loader command, SQL
  probe call, expected output shape.
- That a failing anchor is a signal to investigate the converter / loader /
  build / scan path — *not* a signal to retune the published constant.

### 2. New canonical converter profile

`scripts/qdrant_dbpedia_to_tsv.py` has a new third profile alongside the
two existing gate profiles:

```python
"tqhnsw_real_ann_benchmarks_anchor": SubsetProfile(
    prefix="tqhnsw_real_ann_benchmarks_anchor",
    corpus_rows=990_000,
    query_rows=10_000,
),
```

It reuses the same `_id ascending lexicographic` selection rule, so the
emitted TSV pair and manifest are reproducible from the same parquet that
the rest of the real-corpus lane already loads. The existing
`tqhnsw_real_10k` and `tqhnsw_real_50k` profiles are unchanged (additive
only).

The loader (`scripts/load_real_corpus.py`) needed no changes — it already
handles any prefix that follows the canonical
`<prefix>_{corpus,queries}.tsv` layout.

### 3. New SQL probe `tqhnsw_graph_scan_recall_ann_benchmarks_reference`

Added to `src/lib.rs` in the same `#[cfg(any(test, feature = "pg_test"))]`
block as `tqhnsw_graph_scan_recall_external_summary`. Returns one row:

```
m | ef_search | recall_at_10 | published_recall_at_10 | absolute_delta | within_two_percent
```

Implementation notes:

- The published number lives in two new constants near
  `RECALL_GATE_CONFIGS`:
  - `ANN_BENCHMARKS_ANCHOR_PUBLISHED_RECALL_AT_10: f32 = 0.96082`
  - `ANN_BENCHMARKS_ANCHOR_TOLERANCE: f32 = 0.02`
  Both are commented with the source URL and a pointer to
  `docs/RECALL_ANN_BENCHMARKS_ANCHOR.md`.
- The probe is built on top of `ExternalRecallContext`. The new helper
  `probe_graph_scan_recall_ann_benchmarks_reference_for_relation` delegates
  to the existing `probe_graph_scan_recall_external_summary_for_relation`
  and just extracts `graph_recall_at_10` (field `.4`), so the graph-scan
  logic is not duplicated.
- The `pg_extern` wrapper is the only public surface and follows the same
  shape as `tqhnsw_graph_scan_recall_external_summary`.

### 4. New ignored end-to-end Rust test

`tests/recall_integration.rs` gained one new `#[ignore]`d test:
`ann_benchmarks_anchor_within_tolerance`.

What it does:

- Reads `TQV_ANCHOR_PARQUET` and `TQV_ANCHOR_OUTPUT_DIR` from the
  environment, with optional `TQV_PSQL_BIN`, `TQV_ANCHOR_SKIP_LOAD`, and
  the standard `PG*` libpq env.
- Drives `python3 scripts/qdrant_dbpedia_to_tsv.py --profile
  tqhnsw_real_ann_benchmarks_anchor ...` to produce the TSV pair plus
  manifest.
- Drives `python3 scripts/load_real_corpus.py --prefix
  tqhnsw_real_ann_benchmarks_anchor --m 16` to load the staged TSVs into
  the target database.
- Shells out to `psql -X -A -t -q -c "SELECT ... FROM
  tqhnsw_graph_scan_recall_ann_benchmarks_reference(...)"` and parses the
  one returned row.
- Asserts `|measured - published| <= 0.02` and prints the row to stdout
  when run with `--nocapture`.

The `#[ignore]` attribute is non-negotiable per the task spec: this test
is a manual oracle, not a CI gate, and it must not be promoted to one. The
panic message on failure explicitly tells the operator to investigate
upstream pieces and *not* retune the published constant.

### 5. Cross-link from the primary real-corpus doc

`docs/RECALL_REAL_CORPUS.md` "What This Document Does Not Cover" gained one
bullet pointing to `docs/RECALL_ANN_BENCHMARKS_ANCHOR.md` so a reviewer
landing on the gate doc can find the oracle without searching.

## Design notes followed

- One row, not a sweep.
- Anchor reuses the same `build_source_column = 'source'` build path as
  the primary gate. The new profile lets the existing loader create the
  `tqhnsw_real_ann_benchmarks_anchor_m16_idx` via the same SQL the rest of
  the lane uses.
- Single hardcoded `f32` constant for the published number, with a
  comment pointing at the source URL and the doc — explicitly an oracle,
  not a dynamic comparison.
- Real corpus, no synthetic substitution.

## Evidence

### Lint

```
cargo clippy --all-targets --no-default-features \
    --features 'pg17 pg_test' -- -D warnings
```

is clean on the new branch.

### Pgrx test suite

```
PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17
```

green: 241 + 5 + 10 + 2 + 13 = 271 passed, 0 failed, 28 ignored across all
test binaries. The new `ann_benchmarks_anchor_within_tolerance` test
appears as one of the 28 ignored entries (1 ignored in
`tests/recall_integration.rs` when filtered with `-- ann_benchmarks`).

### New profile is registered

```
$ python3 scripts/qdrant_dbpedia_to_tsv.py --help
  ...
  --profile {tqhnsw_real_10k,tqhnsw_real_50k,tqhnsw_real_ann_benchmarks_anchor}
                        Canonical subset profile to emit.
```

### Anchor probe surface is registered

`tqhnsw_graph_scan_recall_ann_benchmarks_reference(text, text, text, integer,
integer)` lives in the `pg_test` cfg block in `src/lib.rs`, alongside the
other `tqhnsw_graph_scan_recall_external_*` surfaces.

### Manual end-to-end run is deferred

The end-to-end run that actually executes the anchor against the staged
990k corpus and records the first measured `recall@10` requires:

1. The Qdrant DBpedia 1M parquet on local disk (already present per
   review 222 at
   `/home/peter/dev/datasets/qdrant-dbpedia-entities-openai3-text-embedding-3-large-1536-1M`).
2. Free disk for the staged 990k TSV pair.
3. The scratch `pg17` cluster with the `pg_test` build of the extension
   installed (per `docs/RECALL_REAL_CORPUS.md`).
4. A 990k-row hnsw build at `m=16, ef_construct=128`.

Step 4 alone is hours of CPU on the scratch box. This packet lands the
code path and leaves the first measured number for coder-1 to capture
alongside their first DBpedia gate run, which is exactly the
"alongside, so a bad first number is diagnosable" framing in the task
prompt. The `ignored` test command line in the doc is the canonical way
to capture that number when ready.

## Why This Matters

Until this branch lands, the only signal that the real-corpus A4 lane is
measuring something correct is that the gate number "looks plausible". If
the converter mis-orders rows, the loader miscopies a vector dimension,
the build path silently uses the wrong column, or the scan path drops a
neighbor band, the gate would still produce a number — just a wrong one,
with no public reference to spot-check it against.

This packet lands a single, source-controlled, public reference number
the lane can be diff'd against. It is intentionally cheap to add and
intentionally cheap to ignore in CI; its whole job is to be a reviewer
"smell test" the very first time a real-corpus gate number is recorded.

## Out of Scope

- Diagnosing a failing anchor. Per the task spec, if the measured number
  drifts more than 0.02 from `0.96082`, this branch lands the probe red
  and a follow-up review is filed; no fix on this branch.
- Anchor sweep over `(m, ef_search)`. That is task 10054's surface.
- Adding `tqhnsw_real_ann_benchmarks_anchor` as a supported gate profile.
  It is an oracle fixture — the gate fixtures stay `tqhnsw_real_50k` and
  `tqhnsw_real_10k`.
- Integrating the anchor into any CI workflow.

## Files

- `docs/RECALL_ANN_BENCHMARKS_ANCHOR.md` (new)
- `docs/RECALL_REAL_CORPUS.md` (cross-link, +3 lines)
- `scripts/qdrant_dbpedia_to_tsv.py` (new profile, +10 lines)
- `src/lib.rs` (constants, row type, helper, pg_extern wrapper, +77 lines)
- `tests/recall_integration.rs` (ignored end-to-end test, +153 lines)
