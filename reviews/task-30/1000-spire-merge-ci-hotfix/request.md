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
- Replaces shallow moving-branch PR changed-file diffs with immutable PR-base-SHA diffs.
- Records the current clippy baseline explicitly and applies mechanical `cargo fmt` output.

## Validation

- `artifacts/cargo-fmt-check.log` - `cargo fmt --all -- --check` passed.
- `artifacts/cargo-clippy-pg18-bench.log` - `cargo clippy --all-targets --no-default-features --features pg18,bench -- -D warnings` passed.
- `artifacts/cargo-test-ecaz-cli-steprecord.log` - focused `StepRecord` regression test passed.
- `artifacts/cargo-test-no-run-pg18.log` - PG18 no-run compile passed.
- `artifacts/ci-pg18-scope-diff-check.log` - PG18-only CI scope commit has no whitespace errors.
- `artifacts/ci-pg18-clippy-fix-diff-check.log` - PG18 clippy-fix commit has no whitespace errors.
- `artifacts/ci-hosted-portable-rustflags-diff-check.log` - hosted CI portable-RUSTFLAGS commit has no whitespace errors.
- `artifacts/ci-macos-pg18-clippy-fix-diff-check.log` - macOS PG18 clippy-fix commit has no whitespace errors.

Note: a local `cargo +1.95.0 clippy --all-targets --no-default-features --features pg18,bench -- -D warnings` validation was attempted after clearing a stale PG17 cargo process, but it ran too long without diagnostics and was stopped. The authoritative validation for this follow-up is the PR CI rerun on the same pinned toolchain.
