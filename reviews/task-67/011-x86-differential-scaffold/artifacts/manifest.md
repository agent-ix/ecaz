# Task 67 Packet 011 Artifact Manifest

- Head SHA: `a22677eb8a0822196e1af921e819ec0c289560de`
- Task bucket: `reviews/task-67/`
- Packet path: `reviews/task-67/011-x86-differential-scaffold/`
- Timestamp: `2026-05-30T02:15:23Z`
- Lane: x86 RaBitQ SIMD differential-test scaffold
- Fixture: unit-test compile coverage for test-facing sum-query-dequant registry
- Storage format: not applicable
- Rerank mode: not applicable
- Surface isolation: not applicable; no benchmark or SQL surface was run

## Artifacts

### `validation.log`

- Command: `cargo fmt`
- Command: `cargo test -p ecaz task67_sum_query_dequant_for_test_scaffold_registers_expected_kernels --no-run`
- Command: `cargo test -p ecaz task67_sum_query_dequant_for_test_scaffold_matches_scalar_when_available --no-run`
- Command: `git diff --check`
- Result: all passed.
- Key lines cited by `request.md`:
  - `Finished test profile ... target(s) in 3m 06s`
  - `git diff --check` produced no output.

## Limitations

- Runtime execution is intentionally not cited as passing here. The local test
  binary runtime path is still affected by the PostgreSQL `LockBuffer` symbol
  issue observed in prior Task 67 work.
- AVX-512 runtime acceptance cannot be proven on this host because the CPU does
  not expose AVX-512 features.
