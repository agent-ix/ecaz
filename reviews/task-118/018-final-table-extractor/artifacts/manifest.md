# Task 118 Packet 018 Artifact Manifest

- head SHA: `95dda99de3ba2d4d240537106a5ee8ba7041e8d4`
- task bucket: `reviews/task-118/018-final-table-extractor`
- generated: `2026-06-21`
- lane / fixture / storage format / rerank mode: offline final-table
  extraction for Task 118 HNSW source-build and compressed-build suite results.
- isolated surface: consumes `ecaz bench suite` result JSONL from the existing
  one-index-per-prefix suite layout.

## Artifacts

### `task118-final-table.jq`

- purpose: join recall, frontier, score-correlation, and storage result rows by
  scale, format, and build path.
- input: one or more `ecaz bench suite` `results-*.jsonl` files, passed with
  `jq -sr`.
- output: TSV table with the packet 006 final decision columns plus blank
  interpretation columns.

Expected final Intel command:

```bash
jq -sr -f reviews/task-118/018-final-table-extractor/artifacts/task118-final-table.jq \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-10k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-50k-intel.jsonl \
  reviews/task-118/006-final-attribution-matrix/artifacts/results-100k-intel.jsonl
```

### `final-table-extractor-10k-amd-validation.txt`

- purpose: validation output proving the extractor joins the available 10k
  source/compressed rows with packet 016 current-head frontier and
  score-correlation rows.
- input artifacts:
  - `reviews/task-118/006-final-attribution-matrix/artifacts/results-10k.jsonl`
  - `reviews/task-118/006-final-attribution-matrix/artifacts/results-10k-compressed-rerun.jsonl`
  - `reviews/task-118/016-current-head-10k-amd-diagnostics/artifacts/results-10k-frontier-current-head-amd.jsonl`
  - `reviews/task-118/016-current-head-10k-amd-diagnostics/artifacts/results-10k-score-current-head-amd.jsonl`
- expected row count: 6 data rows plus header.
