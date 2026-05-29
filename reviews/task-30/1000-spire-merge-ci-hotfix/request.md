# Review Request: SPIRE Merge CI Hotfix

- Task: Task 30 Phase 13e SPIRE AWS production gap closure
- Code commits:
  - `6bcde972bb9da54b9b083525236b44d0b37b7f3f` - stabilize SPIRE merge CI
  - `2bd73e16be0b30e482ad750b54dc7ab9a7f21e2a` - install pinned Rust CI components
  - `c6bfb0baf4cf01d8f94142fc0f0def8f1eda065f` - work around macOS arm64 `ring` feature assertion
- Packet: `reviews/task-30/1000-spire-merge-ci-hotfix`

## Summary

This hotfix stabilizes the post-merge CI failures from PR #6 without changing SPIRE runtime behavior.

- Fixes the real merge miss by adding `pgoptions: None` to the `StepRecord` test fixture.
- Pins CI stable Rust toolchains to `1.95.0` so workflow behavior does not drift under a floating `stable`.
- Installs required `rustfmt`/`clippy` components for pinned toolchain jobs.
- Applies the macOS arm64 `ring` workaround with `RUSTFLAGS=-C target-feature=-sha3`.
- Installs actual PG17/PG18 server packages in jobs that run `cargo pgrx init`.
- Replaces shallow moving-branch PR changed-file diffs with immutable PR-base-SHA diffs.
- Records the current clippy baseline explicitly and applies mechanical `cargo fmt` output.

## Validation

- `artifacts/cargo-fmt-check.log` - `cargo fmt --all -- --check` passed.
- `artifacts/cargo-clippy-pg18-bench.log` - `cargo clippy --all-targets --no-default-features --features pg18,bench -- -D warnings` passed.
- `artifacts/cargo-test-ecaz-cli-steprecord.log` - focused `StepRecord` regression test passed.
- `artifacts/cargo-test-no-run-pg18.log` - PG18 no-run compile passed.
