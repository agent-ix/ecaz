## Feedback: Real-Corpus Storage-Format Harness

Read `scripts/load_real_corpus.py` (`_index_prefix`, `_index_name`,
`_expected_index_reloptions`, `_build_index_sql`), the gate path in
`scripts/run_real_corpus_recall_scratch.sh`, and
`scripts/tests/test_load_real_corpus_storage_format.py`.

### What's right

- **Shared tables, format-specific indexes.** `<prefix>_corpus` and
  `<prefix>_queries` stay the same across formats; indexes get
  `<prefix>_<storage_format>_m{N}_idx`. That's exactly the right
  factoring — tables are the expensive-to-stage artifact, indexes
  are the cheap-to-add-per-format thing. Operators can now
  benchmark both formats against the same data by adding an index,
  not re-loading a corpus.
- **Reloption assertion matches the build SQL.** `storage_format`
  gets added to both the expected reloption check and the
  `CREATE INDEX` body, conditional on `--storage-format` being
  passed. That keeps pre-flight validation and actual build in
  lockstep.
- **Legacy default path preserved.** Runs without
  `--storage-format` continue to produce `<prefix>_m{N}_idx` with
  no `storage_format` reloption. That's correct — backwards
  compatibility for existing scripts and CI lanes that build
  against the default surface.
- **Focused Python regression tests.** The new
  `test_load_real_corpus_storage_format.py` covers legacy
  preservation, explicit-format naming, reloption conditionality,
  and invalid-format rejection. Narrow and high-value.

### Concerns

1. **Loader behavior under `--storage-format` mismatch with an
   existing index.** If an operator runs `--storage-format
   pq_fastscan` over a corpus where a `turboquant` index already
   exists under a different name, nothing forces them to rebuild
   both. That's a feature (coexistence) but worth documenting
   precisely what the loader does: does it skip if the index
   exists, drop-and-recreate, or error? Current code path isn't
   clear from the packet description.
2. **External gate path derives the fixture prefix from
   `<prefix>_<storage_format>` unconditionally when
   `--storage-format` is set.** If the gate helper has case
   sensitivity or hyphen/underscore expectations, mismatch between
   "what the loader created" and "what the gate queries" would
   surface as zero results. Worth one cross-path test that loads
   then gates without manual prefix overrides.
3. **Docs updated; no verified runbook example.** The
   `docs/RECALL_REAL_CORPUS.md` update describes the new flags but
   a worked example ("here's how I benchmark both formats on the
   same 50k corpus") would be a better landing artifact than a
   flag reference.

### Observation

Operator-harness improvements that make task 15's landing claim
real. Until this, "both formats are first-class" was a SQL-surface
claim; after this, the real-corpus harness can actually prove both
formats on one shared dataset.
