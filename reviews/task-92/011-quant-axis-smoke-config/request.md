# Task 92 Packet 011: Quant Axis Smoke Suite

## Summary

This packet adds a checked-in Task 92 quant-axis smoke suite:

- `crates/ecaz-cli/suites/task92-quant-axis-smoke.json`
- one populated Task 87 LUT32-style TurboQuant cell:
  - `quant=turboquant`
  - `isa=scalar`
  - `kernel_status=valid`
- one explicit Graviton 4 target missing-kernel marker:
  - `quant=rabitq`
  - `isa=sve2`
  - `kernel_status=missing_kernel`

The suite uses normal `latency` steps so dry-run validates the real suite
expansion path. The missing-kernel cell is non-runnable by design and can emit a
`kernel_cell` result row without touching PostgreSQL.

This is a smoke config and marker-result proof, not a real latency benchmark
run. It moves Task 92 Phase 5 closer to acceptance criterion 7; the full
all-quant/all-index matrix remains follow-up work.

## Code

- `26503908cef0` - `Add Task 92 quant axis smoke suite`

## Validation

Artifacts are packet-local under `artifacts/`:

- `artifacts/cargo-test-task92-quant-axis-smoke.log`
  - command: `cargo test -p ecaz-cli commands::bench::suite::tests::parses_task92_quant_axis_smoke_config --no-default-features`
  - result: 1 passed; 0 failed
- `artifacts/dry-run.log`
  - command: `cargo run -p ecaz-cli --no-default-features -- bench suite run --config crates/ecaz-cli/suites/task92-quant-axis-smoke.json --dry-run --manifest-output reviews/task-92/011-quant-axis-smoke-config/artifacts/dry-run-suite-manifest.json`
  - result: dry-run wrote a manifest, printed the populated TurboQuant latency
    command, and printed `kernel_status=missing_kernel` for the RaBitQ SVE2
    cell
- `artifacts/dry-run-suite-manifest.json`
  - populated cell status: `dry-run`
  - missing Graviton 4/SVE2 cell status: `skipped`
- `artifacts/missing-only-run.log`
  - command: `cargo run -p ecaz-cli --no-default-features -- bench suite run --config crates/ecaz-cli/suites/task92-quant-axis-smoke.json --only latency-spire-rabitq-sve2-missing --manifest-output reviews/task-92/011-quant-axis-smoke-config/artifacts/missing-only-suite-manifest.json --results-output reviews/task-92/011-quant-axis-smoke-config/artifacts/missing-only-results.jsonl`
  - result: no benchmark command executed; results file written
- `artifacts/missing-only-results.jsonl`
  - key row: `metric="kernel_cell"`, `quant="rabitq"`,
    `isa="sve2"`, `kernel_status="missing_kernel"`
- `artifacts/git-diff-check.log`
  - command: `git diff --check`
  - result: passed

## Review Notes

The Arm marker is intentionally `isa=sve2` for AWS Graviton 4. No Graviton 3 or
SVE-256 assumption is introduced here.
