# Artifact Manifest

- head SHA: `40c36f73982459f6fec39590482878445b5b187a`
- task bucket: `reviews/task-87/`
- packet path: `reviews/task-87/001-batch-max-scorer/`
- timestamp: `2026-06-08T05:37:39Z`
- lane: Task 87 TurboQuant candidate batching
- fixture: focused Rust unit test
- storage format: TurboQuant no-QJL 4-bit
- rerank mode: not applicable
- isolated one-index-per-table vs shared-table surface: not applicable; no SQL
  index fixture was created

## Artifacts

### `cargo-test-filter-all-targets.log`

- command: `cargo test turboquant_assignment_scorer_uses_no_qjl_4bit_lut_path`
- result: pass
- key lines:
  - `test am::ec_spire::quantizer::tests::turboquant_assignment_scorer_uses_no_qjl_4bit_lut_path ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1989 filtered out; finished in 0.04s`

### `cargo-test-lib-filter.log`

- command: `timeout 180 cargo test --lib turboquant_assignment_scorer_uses_no_qjl_4bit_lut_path`
- result: pass
- key lines:
  - `test am::ec_spire::quantizer::tests::turboquant_assignment_scorer_uses_no_qjl_4bit_lut_path ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1989 filtered out; finished in 0.04s`

### format/static checks

- command: `cargo fmt --check`
- result: pass, with existing stable-rust warnings about unstable
  `imports_granularity` and `group_imports` rustfmt options
- command: `git diff --check`
- result: pass
