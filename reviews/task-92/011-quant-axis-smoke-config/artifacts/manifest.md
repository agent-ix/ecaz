# Task 92 Packet 011 Artifact Manifest

- head SHA: `26503908cef0`
- task bucket: `reviews/task-92`
- packet path: `reviews/task-92/011-quant-axis-smoke-config`
- timestamp: `2026-06-08T21:53:51-07:00`
- lane: Task 92 quant-axis suite smoke
- fixture: `crates/ecaz-cli/suites/task92-quant-axis-smoke.json`
- storage format: TurboQuant populated smoke cell; RaBitQ missing-kernel marker
- rerank mode: not applicable
- table surface: no benchmark tables were created; dry-run and skipped-marker
  result extraction only

## Artifacts

### `artifacts/cargo-test-task92-quant-axis-smoke.log`

- command: `cargo test -p ecaz-cli commands::bench::suite::tests::parses_task92_quant_axis_smoke_config --no-default-features`
- purpose: focused parser/manifest coverage for the checked-in Task 92 smoke
  suite
- key result lines:
  - `test commands::bench::suite::tests::parses_task92_quant_axis_smoke_config ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 410 filtered out; finished in 0.00s`

### `artifacts/dry-run.log`

- command: `cargo run -p ecaz-cli --no-default-features -- bench suite run --config crates/ecaz-cli/suites/task92-quant-axis-smoke.json --dry-run --manifest-output reviews/task-92/011-quant-axis-smoke-config/artifacts/dry-run-suite-manifest.json`
- purpose: end-to-end suite dry-run of the populated TurboQuant cell plus
  missing RaBitQ Graviton 4/SVE2 marker
- key result lines:
  - `latency-spire-turboquant-lut32-scalar-populated -> --database tqvector_bench bench latency`
  - `latency-spire-rabitq-sve2-missing -> kernel_status=missing_kernel`

### `artifacts/dry-run-suite-manifest.json`

- command: produced by the dry-run command above
- purpose: durable manifest evidence for the selected populated and missing
  cells
- key result lines:
  - populated cell: `"quant": "turboquant"`, `"isa": "scalar"`,
    `"kernel_status": "valid"`, `"status": "dry-run"`
  - missing cell: `"quant": "rabitq"`, `"isa": "sve2"`,
    `"kernel_status": "missing_kernel"`, `"status": "skipped"`

### `artifacts/missing-only-run.log`

- command: `cargo run -p ecaz-cli --no-default-features -- bench suite run --config crates/ecaz-cli/suites/task92-quant-axis-smoke.json --only latency-spire-rabitq-sve2-missing --manifest-output reviews/task-92/011-quant-axis-smoke-config/artifacts/missing-only-suite-manifest.json --results-output reviews/task-92/011-quant-axis-smoke-config/artifacts/missing-only-results.jsonl`
- purpose: prove skipped marker cells produce a result row without executing a
  benchmark command
- key result lines:
  - `wrote reviews/task-92/011-quant-axis-smoke-config/artifacts/missing-only-results.jsonl`

### `artifacts/missing-only-suite-manifest.json`

- command: produced by the missing-only command above
- purpose: manifest paired with the missing-only result extraction
- key result lines:
  - missing cell: `"quant": "rabitq"`, `"isa": "sve2"`,
    `"kernel_status": "missing_kernel"`, `"status": "skipped"`

### `artifacts/missing-only-results.jsonl`

- command: produced by the missing-only command above
- purpose: structured result output for skipped missing-kernel cell
- key result lines:
  - `"metric":"kernel_cell"`
  - `"isa":"sve2"`
  - `"kernel_status":"missing_kernel"`
  - `"quant":"rabitq"`

### `artifacts/git-diff-check.log`

- command: `git diff --check`
- purpose: whitespace check for the code and packet diff
- key result lines:
  - `COMMAND_EXIT_CODE="0"`
