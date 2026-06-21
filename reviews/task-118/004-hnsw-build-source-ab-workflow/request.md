# Task 118 Review Request: HNSW Build-Source A/B Workflow

## Summary

This slice adds the workflow needed to test whether HNSW quantized-recall loss is caused by building graph neighbors from compressed/indexed vectors instead of the heap source column.

Code commit under review: `7da9ce8ff8533acfb96979e1e8da9a230f028102`

Changes:

- Added `ecaz corpus load --hnsw-build-source-column <column>` and `--no-hnsw-build-source-column`.
- Kept existing HNSW behavior source-backed by default: `build_source_column = source`.
- Added suite load-step fields for the same options.
- Extended `crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json` to 108 steps: source-build and compressed-build variants for 10k/50k/100k x turboquant/pq_fastscan/rabitq across load, recall, frontier, score correlation, latency, and storage.

## Validation

Artifacts are under `reviews/task-118/004-hnsw-build-source-ab-workflow/artifacts/`.

- `cargo-test-ecaz-cli-hnsw-build-source-ab.log`: `cargo test -p ecaz-cli hnsw -- --nocapture`
  - Result: `21 passed; 0 failed`
- `cargo-check-pg18-pgtest.log`: `cargo check --no-default-features --features pg18,pg_test`
  - Result: completed successfully
- `suite-dry-run.log` and `suite-dry-run-manifest.json`: Task 118 suite dry-run
  - Result: 108 selected steps
  - Confirmed compressed-build load commands include `--no-hnsw-build-source-column` for all 3 scales x 3 formats.

## Review Focus

- CLI semantics: default HNSW load should still build from `source`; compressed-build A/B should omit `build_source_column`.
- Suite semantics: compressed-build prefixes should be isolated and should have matching recall, frontier, score-correlation, latency, and storage measurements.
- Packet evidence: this is workflow validation only. Full 10k/50k/100k benchmark execution remains pending before Task 118 closeout.
