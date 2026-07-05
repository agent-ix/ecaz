# Artifact Manifest: Task 39 Test Quality Lanes

- Head SHA: `80d0fe0c002edd3ba3466d8fe2694b5dbcb59410`
- Task bucket: `reviews/task-39/001-test-quality-lanes`
- Timestamp: `2026-05-18`
- Lane: local script / Makefile wiring validation
- Fixture / storage / rerank: not applicable
- Surface isolation: no live PostgreSQL or corpus fixture used

## `bash-n-hardening.log`

- Command: `bash -n scripts/hardening.sh`
- Result: pass, empty output.

## `bash-n-install-hardening-tools.log`

- Command: `bash -n scripts/install_hardening_tools.sh`
- Result: pass, empty output.

## `make-n-quality-lanes.log`

- Command: `make -n coverage coverage-report mutants MUTANTS_MODULE=src/quant/prod.rs mutants-full flake-hunt`
- Result: pass.
- Key lines:
  - `bash scripts/hardening.sh coverage --output-dir target/quality/coverage`
  - `bash scripts/hardening.sh mutants --file src/quant/prod.rs --output-dir target/quality/mutants --jobs 0`
  - `bash scripts/hardening.sh flake-hunt --seeds 8 --fuzz-seconds 10`

## `mutants-tool-check.log`

- Command: `bash scripts/hardening.sh mutants --file src/quant/prod.rs --output-dir target/quality/mutants --jobs 0`
- Result: expected local skip/fail with exit 127 because `cargo-mutants` is not installed.
- Key line: `missing optional hardening tool: cargo-mutants`
