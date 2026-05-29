# Manifest: SPIRE Merge CI Hotfix

- head SHA: `a20b893b06f839a056912323fd5be71aa1e279e7`
- task bucket: `reviews/task-30`
- packet path: `reviews/task-30/1000-spire-merge-ci-hotfix`
- timestamp: `2026-05-29T21:35:46Z`
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
| `ci-spire-stage-e-remote-timeout-install-permission-failure.log` | `gh api /repos/agent-ix/ecaz/actions/jobs/78574630864/logs` | Stage E `remote_statement_timeout` built successfully, then failed copying `ecaz.control` to `/usr/share/postgresql/18/extension` with permission denied |
| `ci-pg18-install-dir-ownership-diff-check.log` | `git diff --check` | no whitespace errors in the PG18 CI extension-install directory ownership fix |
| `ci-rust-checks-raw-cargo-test-pg-symbol-failure.log` | `gh api /repos/agent-ix/ecaz/actions/jobs/78576285583/logs` | Rust Checks raw `cargo test` failed when the extension crate test binary loaded outside PostgreSQL and could not resolve `CacheRegisterRelcacheCallback` |
| `ci-x86-matrix-raw-cargo-test-pg-symbol-failure.log` | `gh api /repos/agent-ix/ecaz/actions/jobs/78576279464/logs` | x86 PG18 matrix raw `cargo test` failed with the same PostgreSQL backend symbol loader error |
| `ci-host-test-routing-diff-check.log` | `git diff --check` | no whitespace errors in the host-test routing workflow fix |
| `cargo-test-host-side-crates.log` | `cargo test -p ecaz-cloud -p ecaz-fault-injection -p ecaz-sqlgen` | `12 passed; 0 failed` across host-side crate unit tests |
| `cargo-test-no-run-pg18-host-routing.log` | `cargo test --no-run --no-default-features --features pg18` | extension test binaries compiled without host execution |
| `ci-spire-stage-e-remote-timeout-run-dir-cache-collision.log` | `gh api /repos/agent-ix/ecaz/actions/jobs/78578784924/logs` | Stage E `remote_statement_timeout` refused to reuse a cached `target/spire-stage-e-transport-fault-pg18-ci-remote_statement_timeout` run directory |
| `ci-spire-stage-e-run-id-unique-diff-check.log` | `git diff --check` | no whitespace errors in the unique Stage E CI run-id fix |
| `ci-rust-checks-ivf-v1-fixture-format-expectation.log` | `gh api /repos/agent-ix/ecaz/actions/jobs/78579609704/logs` | `ivf_metadata_v1_fixture_decodes` decoded format `1`, while the test expected current writer format `2` |
| `ivf-v1-fixture-expectation-diff-check.log` | `git diff --check` | no whitespace errors in the IVF v1 fixture expectation fix |
| `cargo-fmt-check-ivf-v1-fixture.log` | `cargo fmt --all -- --check` | exited 0 after the IVF v1 fixture expectation fix |
| `cargo-test-ivf-v1-fixture.log` | `cargo test --features bench --test on_disk_fixtures ivf_metadata_v1_fixture_decodes` | focused IVF v1 fixture test passed |
| `cargo-test-on-disk-fixtures-after-ivf-v1.log` | `cargo test --features bench --test on_disk_fixtures` | `test result: ok. 47 passed; 0 failed` |
| `ci-pgrx-pg18-raw-host-lib-symbol-failure.log` | `gh api /repos/agent-ix/ecaz/actions/jobs/78581225390/logs` | `cargo pgrx test pg18` raw-executed the extension crate test binary and failed loading outside PostgreSQL with undefined symbol `LockBuffer` |
| `git-diff-check-pgrx-pg18-preload-ci-final.log` | `git diff --check` | no whitespace errors in the PG18 preload CI workflow fix |
| `cargo-fmt-check-pgrx-pg18-preload-ci-final.log` | `cargo fmt --all -- --check` | exited 0 after the PG18 preload CI workflow fix |
| `cargo-check-ecaz-cli-pgrx-pg18-preload-ci-final.log` | `cargo check -p ecaz-cli` | `Finished dev profile`; only pre-existing `LoadedDistributedPlacementConfig::path` warning remains |
| `cargo-test-ecaz-cli-dev-support-pgrx-paths.log` | `cargo test -p ecaz-cli commands::dev::support::tests` | `test result: ok. 4 passed; 0 failed` |
| `git-diff-check-after-task66-merge-warning-fix.log` | `git diff --check` | no whitespace errors after resolving the task 66 merge conflict |
| `cargo-fmt-check-after-task66-merge-warning-fix.log` | `cargo fmt --all -- --check` | exited 0 after resolving the task 66 merge conflict |
| `cargo-test-no-run-pg18-after-task66-merge-warning-fix.log` | `cargo test --no-run --no-default-features --features pg18` | `Finished test profile` after resolving task 66 and fixing the target-dependent unused-argument warning |
