# Task 118 Packet 017 Artifact Manifest

- head SHA: `812013aed951952550a5e884e3d950870cb0d4f1`
- task bucket: `reviews/task-118/017-intel-closeout-handoff-refresh`
- generated: `2026-06-21`
- lane / fixture / storage format / rerank mode: Intel final closeout handoff
  for 10k/50k/100k source-build and compressed-build HNSW suite lanes.
- isolated surface: the runbook preserves the existing suite shape: one HNSW
  index per loaded prefix.

## Artifacts

### `../010-intel-closeout-runbook/artifacts/intel-closeout-runbook.md`

- purpose: exact operator runbook for the final Intel Task 118 evidence pass.
- change in this checkpoint:
  - adds a full 10k Intel suite command;
  - makes packet 016 explicitly AMD-preview-only;
  - updates status, report, extraction, and commit-scope sections to include
    10k, 50k, and 100k Intel artifacts.

### `../011-final-closeout-audit-template/artifacts/final-closeout-audit-template.md`

- purpose: final quality-control checklist for packet 006 closeout.
- change in this checkpoint:
  - requires 10k Intel artifacts in addition to 50k/100k;
  - updates selected-step expectations from 72 to 108 total selected Intel
    steps across the three scales;
  - updates extraction commands and completion standards to cover 10k/50k/100k.

### `suite-dry-run-10k-intel-shape.log`

- purpose: dry-run validation of the newly added 10k Intel closeout command
  shape.
- command:

```bash
cargo run -p ecaz-cli -- --log-file reviews/task-118/017-intel-closeout-handoff-refresh/artifacts/suite-dry-run-10k-intel-shape.log bench suite run --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json --artifact-dir reviews/task-118/006-final-attribution-matrix/artifacts --manifest-output reviews/task-118/017-intel-closeout-handoff-refresh/artifacts/suite-manifest-dry-run-10k-intel-shape.json --results-output reviews/task-118/017-intel-closeout-handoff-refresh/artifacts/results-dry-run-10k-intel-shape.jsonl --only-tag ec_real_10k --dry-run --allow-debug-backend
```

- selected-step count: `36`
- selected-step kinds:
  - `hnsw-frontier`: `6`
  - `hnsw-score-correlation`: `6`
  - `latency`: `6`
  - `load`: `6`
  - `recall`: `6`
  - `storage`: `6`

### `suite-manifest-dry-run-10k-intel-shape.json`

- purpose: structured dry-run manifest proving the 10k handoff command selects
  the expected full source/compressed 10k suite without running benchmark work.
