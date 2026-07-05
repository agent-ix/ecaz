# Review Request: A4 Real-Corpus Recall Lane

Basis: branch `fix/a4-real-corpus-recall-lane` off `main` after review `217`.

## Why This Packet Exists

Review `217` ruled out the easy escape hatches: the in-repo synthetic generators
— uniform and clustered Gaussian — are not credible substitutes for the
`NFR-003` recall surface. Raw reference HNSW only reaches `26-29% Recall@10` on
them at the gate configuration, so no amount of tqvector tuning is going to
meet the gate as long as the gate runs against the synthetic data.

The next blocker for A4 was therefore methodology, not implementation: tqvector
needed an external-corpus benchmark lane that operates on real `1536`-dim
embeddings, with the same one-time-load / one-time-index-build / repeated-rerun
discipline the synthetic gate already uses.

This packet lands that lane end-to-end.

## What Landed

### 1. Dataset contract — `docs/RECALL_REAL_CORPUS.md`

A standalone contract document. Highlights:

- **Primary dataset**: Qdrant `dbpedia-entities-openai-1M` (OpenAI
  `text-embedding-ada-002` embeddings of DBpedia entities), redistributable per
  the published terms. Total `1,000,000` rows, `1536`-dim, `float32`.
- **Default working subset**: `tqhnsw_real_50k` (`50,000` corpus + `1,000`
  queries) — chosen because that is the surface `NFR-003` declares its targets
  against. A smaller `tqhnsw_real_10k` subset is documented for fast iteration.
- **Local file format**: TSV, one row per vector, two columns
  `<id>\t<json_array>`. No header, UTF-8, Unix line endings. Auditable enough
  to inspect with `head -1`, parseable with a single `split('\t', 1)`.
- The contract is dataset-agnostic: any `1536`-dim corpus that lands in this
  format works. The repo never checks in dataset binaries.

The document also pins the schema, the loader idempotency contract, and the
SQL surfaces the rest of the lane exposes (see below).

### 2. Local loader — `scripts/load_real_corpus.py`

A Python loader that bridges a staged `<basename>_corpus.tsv` /
`<basename>_queries.tsv` pair on disk to Postgres tables tqvector understands.

Shape:

```bash
PGDATABASE=tqvector_bench python3 scripts/load_real_corpus.py \
    --prefix tqhnsw_real_50k \
    --corpus-file /path/to/dbpedia_50k_corpus.tsv \
    --queries-file /path/to/dbpedia_1k_queries.tsv \
    --m 8 16
```

Implementation properties:

- Uses `psql COPY ... FROM STDIN WITH (FORMAT text, DELIMITER E'\t')` for the
  raw bulk insert; rows are streamed line-by-line as `id\t{a,b,c}` real-array
  literals, so the loader avoids per-row SPI overhead and avoids holding the
  full corpus in Python memory.
- After the source column is loaded, runs a single
  `UPDATE ... SET embedding = encode_to_tqvector(source, 4, 42)` to populate
  the encoded `tqvector` column next to the source.
- **Idempotent**: skips reload when the table already exists with rows; skips
  index rebuild when the index already exists with the expected
  `(m, ef_construction, build_source_column)` reloptions. Empty leftovers from
  half-finished previous runs are dropped and reloaded.
- **Identifier hardening**: the `--prefix` is validated against
  `^[a-zA-Z_][a-zA-Z0-9_]*$` before any SQL is composed.
- The script does **not** download datasets. It is a pure local-file path.
  Acquisition of the corpus is the user's responsibility.

The loader creates the contract schema:

```sql
CREATE TABLE <prefix>_corpus (
    id        bigint PRIMARY KEY,
    source    real[] NOT NULL,
    embedding tqvector
);
CREATE TABLE <prefix>_queries (
    id     bigint PRIMARY KEY,
    source real[] NOT NULL
);
```

and then for each requested `m`:

```sql
CREATE INDEX <prefix>_m{m}_idx ON <prefix>_corpus
USING tqhnsw (embedding tqvector_ip_ops)
WITH (m = {m}, ef_construction = 128, build_source_column = 'source');
```

Using `build_source_column = 'source'` means the tqhnsw graph is built on the
raw `source real[]` column rather than the round-tripped quantized embedding.
That separates "is the graph good" from "is the quantizer good" — the same
discipline review `215` used for the source-graph reference baseline.

### 3. External-corpus probe surface — `src/lib.rs`

Three new helpers + two new pg_extern functions, all under the existing
`pg_test`-gated `tests` module:

- `load_external_recall_relation(table_name) -> (Vec<i64>, Vec<Vec<f32>>)` —
  reads `(id, source)` from the loaded corpus or query table, ordered by `id`,
  into Rust-side vectors. The returned ids are kept so the probe can translate
  graph heap tids and exact-quantized id outputs back into the row-index space
  the brute-force ground truth lives in.
- `probe_graph_scan_recall_external_summary_for_relation(corpus_table,
  query_table, index_name, m, ef_search) -> GraphScanRecallExternalSummaryRow`
  — the per-config probe. Loads corpus + queries via the helper above,
  computes brute-force fp32 ground truth from the **actual loaded vectors**
  (not regenerated from a seed — that was the gap that made the existing
  fixture-relation probe unusable for real corpora), runs the graph scan via
  `am::debug_gettuple_scan_heap_tids`, and the exact-quantized comparison via
  `SELECT id FROM <corpus> ORDER BY embedding <#> $1 LIMIT 10`. Builds the
  `ctid -> row index` map by translating through the corpus id column instead
  of assuming `ctid` order matches insertion order.
- `run_graph_scan_recall_gate_from_external(corpus_table, query_table,
  fixture_prefix)` — walks the four `RECALL_GATE_CONFIGS` rows defined at
  `src/lib.rs:711-716` against the `<prefix>_m8_idx` and `<prefix>_m16_idx`
  indexes the loader builds.
- `tqhnsw_graph_scan_recall_external_summary` and
  `tqhnsw_graph_scan_recall_external_gate_report` — pg_extern wrappers that
  return TableIterators over the row types above.

### 4. NFR-003 metric coverage

The new summary row exposes the metrics `NFR-003` calls out, not just
Recall@10:

```
m | ef_search | corpus_rows | query_count
  | graph_recall_at_10 | graph_recall_at_100
  | ndcg_at_10 | mean_abs_score_error | spearman_rho_at_10
  | exact_quantized_recall_at_10
  | graph_below_exact_queries | worst_exact_gap
```

`graph_recall_at_100` is computed against a wider `RECALL_K * 10 = 100`-band
brute-force truth from the same loaded corpus, so the NFR-003 wider-recall
column comes for free. NDCG@10 and Spearman are implemented in
row-index space against the predicted top-10. MAE compares the fp32 truth
scores to the fp32 scores of the predicted top-10 (re-dot-producting against
the loaded source vectors), which keeps the metric independent of the
quantizer noise floor.

### 5. Smoke test — `test_tqhnsw_graph_scan_recall_external_smoke_500`

A new ignored `pg_test` (`src/lib.rs`) that exercises the entire external
lane against a tiny synthetic dataset that the contract schema accepts:

- materializes the `<prefix>_corpus` / `<prefix>_queries` tables and the
  `<prefix>_m8_idx` / `<prefix>_m16_idx` indexes via the same SQL the loader
  emits (`build_source_column = 'source'`)
- runs `probe_graph_scan_recall_external_summary_for_relation` once and
  asserts the row has sane bounds (`graph_recall_at_10 ∈ [0, 1]`, `spearman ∈
  [-1, 1]`, NDCG/MAE non-negative)
- reruns the same probe a second time against the same loaded tables and
  asserts the summary row is **byte-identical** to the first run — that is the
  reusable-fixture contract the lane depends on
- runs `run_graph_scan_recall_gate_from_external` and asserts every entry in
  `RECALL_GATE_CONFIGS` is emitted, that targetless rows are reported as
  passing, and that the gated row's recall is in `[0, 1]`

The smoke test is intentionally not asserting a specific recall number against
the smoke fixture: a `500 x 1536` uniformly-random corpus is dominated by the
quantizer noise floor, so any concrete threshold would just relitigate the
synthetic-fixture contradiction the lane was built to solve. The real recall
gate is `tqhnsw_graph_scan_recall_external_gate_report` against the staged
DBpedia corpus.

Result, `pg17`:

```
test tests::pg_test_tqhnsw_graph_scan_recall_external_smoke_500 ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 247 filtered out;
finished in 163.57s
```

The runtime is dominated by `INSERT ... VALUES` row-at-a-time corpus loading
plus two `CREATE INDEX` calls at `m=8` / `m=16`; the probe / rerun / gate
phases are cheap by comparison.

## How to Use the Lane End-to-End

1. Stage the corpus + query files locally per `docs/RECALL_REAL_CORPUS.md`.
2. Load them into the database:
   ```bash
   PGDATABASE=tqvector_bench python3 scripts/load_real_corpus.py \
       --prefix tqhnsw_real_50k \
       --corpus-file /path/to/dbpedia_50k_corpus.tsv \
       --queries-file /path/to/dbpedia_1k_queries.tsv \
       --m 8 16
   ```
3. Run the gate report:
   ```sql
   SELECT * FROM tqhnsw_graph_scan_recall_external_gate_report(
       'tqhnsw_real_50k_corpus',
       'tqhnsw_real_50k_queries',
       'tqhnsw_real_50k'
   );
   ```
   This emits one row per A4 configuration with the same shape the synthetic
   gate report uses, so existing reporting tooling continues to work.
4. For NFR-003 detail per configuration, run:
   ```sql
   SELECT * FROM tqhnsw_graph_scan_recall_external_summary(
       'tqhnsw_real_50k_corpus',
       'tqhnsw_real_50k_queries',
       'tqhnsw_real_50k_m8_idx',
       8,
       128
   );
   ```

The loader is idempotent, so reruns of step 2 are safe and skip the expensive
index build.

## What This Packet Does NOT Do

- It does **not** include actual DBpedia recall numbers. The corpus is staged
  out-of-band by the user; the repo never checks in dataset binaries. The
  first real run is the user's call.
- It does **not** retire the synthetic fixture-backed gate. That gate remains
  useful for runtime / graph invariant debugging, just not as the credibility
  surface for `NFR-003`.
- It does **not** change `RECALL_GATE_CONFIGS`. The gate target is still
  `Recall@10 ≥ 0.89` at `m=8 / ef=128`, matching the synthetic gate.

## Files Changed

- `docs/RECALL_REAL_CORPUS.md` — new dataset contract
- `scripts/load_real_corpus.py` — new local-file loader
- `src/lib.rs` — new external-corpus probes, summary row type, two pg_extern
  wrappers, and smoke test
- `plan/tasks/12-real-corpus-recall.md` — task subtasks marked complete

## Validation

- `cargo check --features pg17` — clean
- `cargo check --features pg17 --tests` — clean
- `cargo check --features "pg17 pg_test"` — clean
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo test --no-default-features --features 'pg17 pg_test' test_tqhnsw_graph_scan_recall_external_smoke_500 -- --ignored --nocapture` — passes (`163.57s`, single test)

## Open Question for the Reviewer

The lane is dataset-agnostic by construction, but the gate target itself —
`Recall@10 ≥ 0.89` at `m=8 / ef=128` — was tuned against the synthetic fixture
generation that review `217` showed is not a credible NFR-003 surface. Once
the first real DBpedia run happens, the reviewer should decide whether the
gate target stays as-is, gets re-tuned to match published DBpedia recall
numbers from comparable libraries, or stays as a hard NFR floor. This packet
intentionally leaves that decision to the first real run.
