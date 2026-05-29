# Review Request: SPIRE Merge CI Hotfix

- Task: Task 30 Phase 13e SPIRE AWS production gap closure
- Code commits:
  - `6bcde972bb9da54b9b083525236b44d0b37b7f3f` - stabilize SPIRE merge CI
  - `2bd73e16be0b30e482ad750b54dc7ab9a7f21e2a` - install pinned Rust CI components
  - `c6bfb0baf4cf01d8f94142fc0f0def8f1eda065f` - work around macOS arm64 `ring` feature assertion
  - `fea9c59f357d44da1db5e705b6321bbc1269750c` - limit PR CI to PG18 targets
  - `113c9293df000b664f6860a1b59a49e5f92f4871` - fix PG18 clippy findings from the pinned-toolchain CI rerun
  - `18cbbbccd0ab859a94ef20ca684d59ce4ed7961d` - use portable CPU flags on hosted CI runners
  - `f316c0de91370ec0caca64c015bf7caf2e5dc7b2` - fix macOS PG18 clippy findings in aarch64-only RabitQ code
  - `a812112b52894d85d2c8828d0541030bb4e7c2e1` - fix arm PG18 CI findings
  - `109d4cc75d44a78b4b1d8e42cd2e63348ed5c87e` - fix portable `StringInfo` byte pointer casts
  - `609cf836670833480430cb0cf02016a180c0a846` - narrow the `StringInfo` no-cast target split to Linux-style aarch64
  - `c65a3508fa66eacb7d99de365ef4986374b9fed9` - repair the careful coverage harness after current source-shape drift
  - `7a9f08b956b02f0b5c433ee2e80ffac9171ad480` - allow PG18 CI extension installs against apt-owned PostgreSQL directories
  - `3e6395fa3e0f44e6aa826b956c445acbe61a26bb` - route host-side CI tests away from raw execution of the PostgreSQL extension crate
  - `7854ff96cf37b410ca417a01fbfe976f6fc05b9c` - make Stage E CI run directories unique per GitHub run attempt
- Packet: `reviews/task-30/1000-spire-merge-ci-hotfix`

## Summary

This hotfix stabilizes the post-merge CI failures from PR #6 without changing SPIRE runtime behavior.

- Fixes the real merge miss by adding `pgoptions: None` to the `StepRecord` test fixture.
- Pins CI stable Rust toolchains to `1.95.0` so workflow behavior does not drift under a floating `stable`.
- Installs required `rustfmt`/`clippy` components for pinned toolchain jobs.
- Applies the macOS arm64 `ring` workaround with `RUSTFLAGS=-C target-feature=-sha3`.
- Installs actual PG18 server packages in jobs that run PG18 `cargo pgrx init`.
- Limits blocking PR CI to PG18, which is the current target; the PG17 pgrx job is manual-only and was not debugged locally.
- Fixes PG18 clippy findings reported by the pinned `1.95.0` CI rerun: dead-code allowances for Hadamard test helpers and key-based sort suggestions in the CustomScan test.
- Overrides hosted CI jobs away from repo-local `target-cpu=native` so build scripts do not SIGILL on heterogeneous GitHub x86 runners.
- Fixes macOS/aarch64 PG18 clippy findings in RabitQ doc comments and tests.
- Restores required macOS pgrx linker flags when overriding `RUSTFLAGS`, and removes an aarch64 PG18 clippy unnecessary cast.
- Replaces shallow moving-branch PR changed-file diffs with immutable PR-base-SHA diffs.
- Records the current clippy baseline explicitly and applies mechanical `cargo fmt` output.
- Keeps `pq_getmsgbytes` byte reads portable across PG18 CI targets: Linux-style aarch64 keeps the native `u8` pointer while x86 and macOS cast PostgreSQL's `c_char` pointer to `u8`.
- Restores the `hardening/careful` coverage crate against current storage, SPIRE, DiskANN, and IVF helper shapes so the Test Quality Coverage job can compile the same coverage harness again.
- Fixes the PG18 Stage E CI setup by making apt-owned PG18 extension install directories writable by the runner before fixture scripts call `cargo pgrx install`; this addresses a setup permission failure, not SPIRE runtime behavior.
- Replaces raw host execution of the `ecaz` extension crate test binary with host-side crate tests plus PG18 extension test compilation; actual backend execution remains in `cargo pgrx test pg18`.
- Avoids Stage E fixture run-directory collisions when GitHub restores cached `target/` contents by using a run-id that includes the GitHub run id and attempt.

## Validation

- `artifacts/cargo-fmt-check.log` - `cargo fmt --all -- --check` passed.
- `artifacts/cargo-clippy-pg18-bench.log` - `cargo clippy --all-targets --no-default-features --features pg18,bench -- -D warnings` passed.
- `artifacts/cargo-test-ecaz-cli-steprecord.log` - focused `StepRecord` regression test passed.
- `artifacts/cargo-test-no-run-pg18.log` - PG18 no-run compile passed.
- `artifacts/ci-pg18-scope-diff-check.log` - PG18-only CI scope commit has no whitespace errors.
- `artifacts/ci-pg18-clippy-fix-diff-check.log` - PG18 clippy-fix commit has no whitespace errors.
- `artifacts/ci-hosted-portable-rustflags-diff-check.log` - hosted CI portable-RUSTFLAGS commit has no whitespace errors.
- `artifacts/ci-macos-pg18-clippy-fix-diff-check.log` - macOS PG18 clippy-fix commit has no whitespace errors.
- `artifacts/ci-arm-pg18-fix-diff-check.log` - arm PG18 CI-fix commit has no whitespace errors.
- `artifacts/ci-string-info-ptr-cast-diff-check.log` - portable `StringInfo` pointer-cast commit has no whitespace errors.
- `artifacts/cargo-fmt-check-string-info.log` - `cargo fmt --all -- --check` passed after the `StringInfo` fix.
- `artifacts/cargo-check-pg18-string-info.log` - `cargo check --no-default-features --features pg18` passed after the `StringInfo` fix.
- `artifacts/ci-string-info-target-split-diff-check.log` - narrowed `StringInfo` target-split commit has no whitespace errors.
- `artifacts/cargo-fmt-check-string-info-target-split.log` - `cargo fmt --all -- --check` passed after narrowing the target split.
- `artifacts/cargo-check-pg18-string-info-target-split.log` - `cargo check --no-default-features --features pg18` passed after narrowing the target split.
- `artifacts/ci-careful-coverage-harness-diff-check.log` - careful coverage harness repair commit has no whitespace errors.
- `artifacts/cargo-test-careful-hardening-lib.log` - `cargo test --manifest-path hardening/careful/Cargo.toml --lib` passed with `573 passed`.
- `artifacts/cargo-check-pg18-careful-harness.log` - `cargo check --no-default-features --features pg18` passed after the careful harness repair.
- `artifacts/make-coverage-missing-llvm-cov.log` - `make coverage` was attempted locally but could not run because this environment lacks `cargo-llvm-cov`; CI installs that tool before running coverage.
- `artifacts/ci-spire-stage-e-remote-timeout-install-permission-failure.log` - CI failure log showing the Stage E `remote_statement_timeout` job built successfully then failed copying `ecaz.control` to `/usr/share/postgresql/18/extension`.
- `artifacts/ci-pg18-install-dir-ownership-diff-check.log` - PG18 CI extension-install directory ownership fix has no whitespace errors.
- `artifacts/ci-rust-checks-raw-cargo-test-pg-symbol-failure.log` - CI failure log showing raw `cargo test` failed at process load with undefined PostgreSQL backend symbol `CacheRegisterRelcacheCallback`.
- `artifacts/ci-x86-matrix-raw-cargo-test-pg-symbol-failure.log` - x86 PG18 matrix failure log with the same raw extension test-binary loader failure.
- `artifacts/ci-host-test-routing-diff-check.log` - host-test routing workflow fix has no whitespace errors.
- `artifacts/cargo-test-host-side-crates.log` - `cargo test -p ecaz-cloud -p ecaz-fault-injection -p ecaz-sqlgen` passed locally.
- `artifacts/cargo-test-no-run-pg18-host-routing.log` - `cargo test --no-run --no-default-features --features pg18` passed locally for the extension test compile path.
- `artifacts/ci-spire-stage-e-remote-timeout-run-dir-cache-collision.log` - CI failure log showing Stage E refused to reuse a cached `target/spire-stage-e-transport-fault-pg18-ci-remote_statement_timeout` directory.
- `artifacts/ci-spire-stage-e-run-id-unique-diff-check.log` - unique Stage E CI run-id fix has no whitespace errors.

Note: a local `cargo +1.95.0 clippy --all-targets --no-default-features --features pg18,bench -- -D warnings` validation was attempted after clearing a stale PG17 cargo process, but it ran too long without diagnostics and was stopped. The authoritative validation for this follow-up is the PR CI rerun on the same pinned toolchain.
