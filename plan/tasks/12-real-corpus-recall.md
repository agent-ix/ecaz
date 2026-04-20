# Task 12: Real-Corpus Recall Validation

Status: done for `v0.1` A4 signoff — actual parquet is fetched, canonical `10K` / `50K` subsets are staged, real `10K` passes strongly, and broader real `50K` gate slices also pass comfortably on `main`.

## Scope

Add the real-corpus benchmark path needed to evaluate A4 on a dataset consistent with
`NFR-003` rather than only the current in-repo synthetic generators.

This task is not about weakening the recall target. It is about measuring the existing gate on a
credible dataset:

- DBpedia OpenAI embeddings, or
- a clearly documented equivalent real `1536`-dimensional embedding corpus

## Why This Exists

Recent A4 investigation changed the problem statement:

- the deterministic synthetic fixtures are reproducible, but they are no longer credible gate
  surfaces
- raw reference `hnsw-rs` on the same synthetic workload only reaches:
  - `29.0%` Recall@10 on uniform `10k` at `m=8, ef_search=128`
  - `26.0%` Recall@10 on the current clustered `10k` at `m=8, ef_search=128`
  - `66.5%` Recall@10 on uniform `10k` even at `m=16, ef_search=200`
- `NFR-003` already says recall benchmarks SHALL run against DBpedia OpenAI embeddings or
  equivalent

So A4 had to be re-grounded on benchmark methodology as well as implementation correctness; this
task is the lane that resolved that ambiguity on `main`.

## Subtasks

- [x] **Dataset contract.** Documented in `docs/RECALL_REAL_CORPUS.md`. Primary: Qdrant
  `dbpedia-entities-openai3-text-embedding-3-large-1536-1M`; default working subset
  `ec_hnsw_real_50k` (50k corpus, 1k queries). Local file format: TSV with
  `<id>\t<json_array>` columns, no header.
- [x] **Canonical subset rule.** `scripts/qdrant_dbpedia_to_tsv.py` pins the default subsets by
  sorting the full parquet release by the source parquet id column ascending (currently `_id`
  lexicographic for the Qdrant/Hugging Face release), then taking rows `[0, 49_999]` / `[50_000,
  50_999]` for `ec_hnsw_real_50k` and `[0, 9_999]` / `[10_000, 10_199]` for `ec_hnsw_real_10k`.
- [x] **Manifest contract.** The converter emits `<prefix>_manifest.json` with SHA-256 digests,
  counts, id ranges, dimensionality, and selection metadata. `scripts/load_real_corpus.py`
  auto-discovers and verifies the sibling manifest (or takes `--manifest-file` explicitly) before
  loading, refusing mismatches unless `--allow-manifest-mismatch` is passed.
- [x] **One-shot scratch helper.** `scripts/prepare_real_corpus_scratch.sh` chains canonical
  parquet conversion into the existing scratch-cluster loader so the first local DBpedia run does
  not depend on a manual multi-step copy/paste sequence.
- [x] **Local loader path.** `scripts/load_real_corpus.py` ingests `<basename>_corpus.tsv` /
  `<basename>_queries.tsv` via `psql COPY ... FROM STDIN`, then encodes the `embedding tqvector`
  column with `encode_to_tqvector(source, 4, 42)`. Idempotent: skips reload when the table is
  already populated and skips index rebuild when reloptions match.
- [x] **External-corpus relation seam.** Added
  `probe_graph_scan_recall_external_summary_for_relation` and the `ec_hnsw_graph_scan_recall_external_summary`
  pg_extern. Reads `(id, source)` from the loaded tables, builds fp32 truth from the actual loaded
  vectors (not regenerated from a seed), runs the graph scan via `am::debug_gettuple_scan_heap_tids`,
  and compares against `ORDER BY embedding <#> $1`.
- [x] **Reusable fixture flow.** Loader idempotency and the smoke test both rerun the probe twice
  against the same loaded tables and assert the summary is byte-identical. The flow matches the
  one-time-load / one-time-index-build / repeated-rerun discipline of the synthetic gate.
- [x] **A4 rerun on real data.** `ec_hnsw_graph_scan_recall_external_gate_report` walks the four
  `RECALL_GATE_CONFIGS` rows — `(8, 40, None)`, `(8, 128, Some(0.89))`, `(8, 200, None)`,
  `(16, 200, None)` — against the `<prefix>_m{8,16}_idx` indexes built by the loader. The actual
  DBpedia run is staged by the user out-of-band per `docs/RECALL_REAL_CORPUS.md`.
- [x] **Reporting surface.** `ec_hnsw_graph_scan_recall_external_summary` returns
  `(m, ef_search, corpus_rows, query_count, graph_recall_at_10, graph_recall_at_100, ndcg_at_10,
  mean_abs_score_error, spearman_rho_at_10, exact_quantized_recall_at_10, graph_below_exact_queries,
  worst_exact_gap)` per call. The gate report adds the explicit `passes_gate` column.
- [x] **A4 closeout evidence.** Canonical real `10K` and broader real `50K` signoff evidence is
  recorded in review packets `223` through `226`, closing A4 on the real-corpus surface required
  by `NFR-003`.

## Owns

- `NFR-003` real-dataset methodology for A4 and later benchmark reporting
- The "DBpedia OpenAI embeddings or equivalent" execution lane implied by the spec

## Dependencies

- Task 05 / A4 probe and debug surfaces on `main`
- Task 10 benchmark/report infrastructure

## Unblocks

- A4 closeout on a credible dataset
- Final `NFR-003` benchmark reporting
- Any project decision about whether the current A4 failure is an implementation defect or a
  synthetic-fixture mismatch

## Deliverables

- A documented local dataset contract
- A deterministic parquet -> TSV conversion recipe for the canonical subsets
- A manifest/hash contract that makes the staged subset reproducible across reruns
- A loader script / SQL path for corpus and query tables
- Reusable relation-backed recall measurement on external data
- A benchmark report or review packet with the real-corpus A4 signoff results

## Primary Tests

- Loader smoke test on a tiny external sample
- Stable rerun of the external-corpus recall report without corpus rebuild
- A4 result table on the chosen real corpus

## Notes

- Deterministic synthetic fixtures remain useful for debugging graph/runtime invariants. They are
  not retired by this task.
- Keep large datasets out of the repository. Check in metadata, loader logic, and benchmark
  documentation, not the corpus itself.
- Prefer local reproducibility over cleverness: explicit file contract, explicit query split,
  explicit seeds or row selections where applicable.
- This task should reuse the current SQL/debug probes rather than invent a second benchmark stack.
