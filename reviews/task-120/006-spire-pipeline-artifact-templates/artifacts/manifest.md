# Task 120 / 006 SPIRE Pipeline Artifact Template Artifacts

- head SHA: `8225571c2d98c8164b56a0355b79826809a1fb9d`
- task bucket: `reviews/task-120/006-spire-pipeline-artifact-templates`
- lane: local validation
- fixture: not applicable; suite runner path-template coverage only
- storage format: not applicable
- rerank mode: SPIRE pipeline benchmark diagnostics
- isolated one-index-per-table vs shared-table surface: not applicable
- timestamp: `2026-06-21T15:21:30Z`

## Artifacts

### `cargo-fmt-check.log`

- command: `cargo fmt --check`
- result: passed
- key line: `Script done on 2026-06-21 08:20:26-07:00 [COMMAND_EXIT_CODE="0"]`
- note: stable rustfmt emitted the repo's existing unstable-option warnings.

### `cargo-test-ecaz-cli-spire-pipeline.log`

- command: `cargo test -p ecaz-cli spire_pipeline`
- result: passed
- key lines:
  - `running 23 tests`
  - `test commands::bench::suite::tests::artifact_dir_templates_rewrite_spire_pipeline_paths ... ok`
  - `test commands::bench::suite::tests::expands_spire_pipeline_with_production_profile ... ok`
  - `test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 388 filtered out; finished in 0.00s`
  - `Script done on 2026-06-21 08:21:18-07:00 [COMMAND_EXIT_CODE="0"]`

### `git-diff-check.log`

- command: `git diff --check`
- result: passed
- key line: `Script done on 2026-06-21 08:21:30-07:00 [COMMAND_EXIT_CODE="0"]`
