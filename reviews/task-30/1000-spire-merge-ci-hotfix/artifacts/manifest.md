# Manifest: SPIRE Merge CI Hotfix

- head SHA: `c65a3508fa66eacb7d99de365ef4986374b9fed9`
- task bucket: `reviews/task-30`
- packet path: `reviews/task-30/1000-spire-merge-ci-hotfix`
- timestamp: `2026-05-29T19:42:25Z`
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
| `ci-macos-pg18-clippy-fix-diff-check.log` | `git show --check --stat --oneline HEAD` | no whitespace errors in the macOS PG18 clippy-fix commit |
| `ci-arm-pg18-fix-diff-check.log` | `git show --check --stat --oneline HEAD` | no whitespace errors in the arm PG18 CI-fix commit |
| `ci-string-info-ptr-cast-diff-check.log` | `git show --check --stat --oneline HEAD` | no whitespace errors in the portable StringInfo pointer-cast commit |
| `cargo-fmt-check-string-info.log` | `cargo fmt --all -- --check` | exited 0 after the StringInfo pointer-cast fix |
| `cargo-check-pg18-string-info.log` | `cargo check --no-default-features --features pg18` | `Finished dev profile` |
| `ci-string-info-target-split-diff-check.log` | `git show --check --stat --oneline HEAD` | no whitespace errors in the narrowed StringInfo target-split commit |
| `cargo-fmt-check-string-info-target-split.log` | `cargo fmt --all -- --check` | exited 0 after narrowing the StringInfo target split |
| `cargo-check-pg18-string-info-target-split.log` | `cargo check --no-default-features --features pg18` | `Finished dev profile` |
| `ci-careful-coverage-harness-diff-check.log` | `git show --check --stat --oneline HEAD` | no whitespace errors in the careful coverage harness repair commit |
| `cargo-test-careful-hardening-lib.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib` | `test result: ok. 573 passed; 0 failed` |
| `cargo-check-pg18-careful-harness.log` | `cargo check --no-default-features --features pg18` | `Finished dev profile` |
| `make-coverage-missing-llvm-cov.log` | `make coverage` | local run stopped before coverage because `cargo-llvm-cov` is not installed in this environment |
