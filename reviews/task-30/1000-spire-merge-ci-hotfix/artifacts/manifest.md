# Manifest: SPIRE Merge CI Hotfix

- head SHA: `18cbbbccd0ab859a94ef20ca684d59ce4ed7961d`
- task bucket: `reviews/task-30`
- packet path: `reviews/task-30/1000-spire-merge-ci-hotfix`
- timestamp: `2026-05-29T18:23:24Z`
- lane: post-merge CI stabilization
- fixture/storage/rerank mode: not applicable
- isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

| Artifact | Command | Key result |
| --- | --- | --- |
| `cargo-fmt-check.log` | `cargo fmt --all -- --check` | exited 0 |
| `cargo-clippy-pg18-bench.log` | `cargo clippy --all-targets --no-default-features --features pg18,bench -- -D warnings` | `Finished dev profile` |
| `cargo-test-ecaz-cli-steprecord.log` | `cargo test -p ecaz-cli result_rows_include_suite_context_fields` | `1 passed; 0 failed` |
| `cargo-test-no-run-pg18.log` | `cargo test --no-run --no-default-features --features pg18` | `Finished test profile` and emitted PG18 test executables |
| `ci-pg18-scope-diff-check.log` | `git show --check --stat --oneline HEAD` | no whitespace errors in the PG18-only CI scope commit |
| `ci-pg18-clippy-fix-diff-check.log` | `git show --check --stat --oneline HEAD` | no whitespace errors in the PG18 clippy-fix commit |
| `ci-hosted-portable-rustflags-diff-check.log` | `git show --check --stat --oneline HEAD` | no whitespace errors in the hosted-CI portable RUSTFLAGS commit |
