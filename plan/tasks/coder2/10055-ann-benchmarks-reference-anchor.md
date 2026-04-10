# Task: ann-benchmarks Reference Anchor Probe

Motivation: Review 218 item 7 flagged that the real-corpus A4 lane has no
external oracle. If `build_external_recall_context` or the Qdrant parquet
converter (`scripts/qdrant_dbpedia_to_tsv.py`) has a subtle bug, every
real-corpus gate run will be silently wrong and we will have no published
reference number to catch it. Reviews 220/221/222 explicitly deferred this
item. With the canonical loader path now proven on the real DBpedia corpus,
this is the right moment to land the anchor: coder-1 is about to record the
first DBpedia gate number and we want an independent "we match a published
number" result alongside it, so a bad first number is diagnosable.
Priority: batch 1
Status: ready

## Prompt

Stand up a one-time reference probe that anchors the real-corpus lane against
a published ann-benchmarks-style number on the same dataset family. The probe
should be `#[ignore]`d so it never runs in CI — it is a manual oracle, not a
gate.

### Step 1 — pick and document the anchor

Add `docs/RECALL_ANN_BENCHMARKS_ANCHOR.md` that states, in one page:

- Which published result we are anchoring against. Pick one concrete row
  from a credible public benchmark that uses the same or an
  architecturally-compatible corpus family to the Qdrant
  `dbpedia-entities-openai3-text-embedding-3-large-1536-1M` release. Good
  candidates (in preference order):
  1. A published hnswlib or FAISS-HNSW row from the Qdrant benchmark
     repo against the same DBpedia-1M / openai / 1536 split. Cite the
     specific file / commit / table row in the doc.
  2. A published ann-benchmarks row for `dbpedia-openai-1000k-angular`
     (the ann-benchmarks repackaging). Note the distance metric.
  3. If neither is reachable, a published hnswlib row on
     `glove-100-angular` at `m=16 ef_construction=200` — state
     explicitly that this is a dimensionality proxy, not a corpus
     match.
- The exact `(m, ef_construction, ef_search)` the anchor reports against, and
  its published `recall@10`. Copy the numeric cell verbatim — no rounding,
  no interpretation.
- The absolute URL or git commit pin for the published table. Prefer a
  permalink (commit SHA or release tag) so the anchor is stable.
- One paragraph on why this row and not another. "It is the most widely
  reproduced hnsw row on a public embedding corpus at 1536-ish dims" is
  enough.

### Step 2 — load the anchor corpus

Extend `scripts/qdrant_dbpedia_to_tsv.py` with a third profile that covers
the anchor's row count:

- If the anchor is against the full 1M Qdrant corpus at 10k queries, add a
  `tqhnsw_real_ann_benchmarks_anchor` profile with the published row count
  and query count. If the anchor is against ann-benchmarks' 1000k/10k
  split, match that.
- Reuse the same canonical selection rule (`_id` ascending lexicographic,
  global sorted row index as the emitted TSV id) so the anchor output is
  reproducible from the same parquet.
- The profile must be additive — do not change existing
  `tqhnsw_real_50k` / `tqhnsw_real_10k` behavior.

The loader (`scripts/load_real_corpus.py`) already handles any prefix that
matches the canonical `<prefix>_{corpus,queries}.tsv` layout. No loader
changes should be needed.

### Step 3 — add the ignored probe

Add `tests.tqhnsw_graph_scan_recall_ann_benchmarks_reference(
    corpus_table text,
    query_table text,
    index_name text,
    m integer,
    ef_search integer
)` to `src/lib.rs` in the same `#[cfg(any(test, feature = "pg_test"))]`
block as `tqhnsw_graph_scan_recall_external_summary`.

Returns one row with columns:

- `m integer`
- `ef_search integer`
- `recall_at_10 float`              — measured on this build
- `published_recall_at_10 float`    — the anchor number from the doc
- `absolute_delta float`            — measured - published
- `within_two_percent bool`         — `|delta| <= 0.02`

The `published_recall_at_10` should be a constant declared near the function
and cross-referenced to `docs/RECALL_ANN_BENCHMARKS_ANCHOR.md`. Hardcoded
number is correct here — this is explicitly an oracle, not a dynamic
comparison.

Build the probe on top of `ExternalRecallContext` — reuse
`build_external_recall_context` and
`probe_graph_scan_recall_external_summary_for_context` rather than
duplicating the graph-scan logic.

Additionally expose a thin Rust integration test in
`tests/recall_integration.rs` that is `#[ignore]`d by default and runs the
full build + load + probe end-to-end. Its only job is to fail loudly if the
measured number drifts more than 0.02 from the published anchor. The ignore
attribute is non-negotiable: this test will not run in CI and must not be
promoted to a gate.

### Step 4 — record the first measured number

Commit the first measured result alongside the code in the review packet.
Do not check in the parquet or TSV binaries, but do commit the generated
`<prefix>_manifest.json` for the anchor run so future reviewers can verify
they are measuring against the same staged subset.

## Design notes

- The anchor is a one-time sanity check, not a continuous metric. Resist
  the temptation to add four anchor configs or to parameterize over
  `(m, ef_search)` grids — that is task 10054's job.
- The anchor must use the same `build_source_column = 'source'` build path
  as the primary gate (`docs/RECALL_REAL_CORPUS.md:147-164`). Different
  build paths invalidate the comparison.
- If the anchor's published number is against cosine and our default is
  inner product on unit-normalized vectors, explicitly call that
  equivalence out in the doc. The loader already logs unit-norm stats
  (`scripts/load_real_corpus.py` `VectorNormStats`), so the equivalence
  is verifiable at load time.
- Do not swap the anchor corpus for a synthetic one to make the probe
  cheaper. The whole point of the anchor is that it is on real data.

## Out of scope

- Diagnosing a failing anchor. If the measured number is off by more than
  0.02, land the probe red and file a follow-up review. Do not fix the
  primary build on this branch.
- Integrating the anchor into any CI workflow.
- Adding the anchor's row counts as a supported gate profile. It is an
  oracle fixture; the gate fixtures stay `tqhnsw_real_50k` and
  `tqhnsw_real_10k`.

## Validate

```bash
cargo clippy --all-targets --no-default-features --features 'pg17 pg_test' -- -D warnings
cargo pgrx test pg17
```

Then, with the anchor corpus staged locally, run the ignored probe manually
and capture the output in the review packet:

```bash
cargo test --features 'pg17 pg_test' --no-default-features \
    tqhnsw_graph_scan_recall_ann_benchmarks_reference -- --ignored --nocapture
```

Branch from current upstream main. Push branch for review.
