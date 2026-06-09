# Task 91 Review Request: Rust Checks Clippy Cleanup

## Summary

This checkpoint fixes the Rust Checks failures observed after Packet 016:

- replaced `std::sync::LazyLock` in block-kernel counter storage with `OnceLock::get_or_init`, preserving the existing counter vector allocation/indexing while staying compatible with the repository MSRV
- replaced DiskANN binary-sidecar byte-buffer capacity calculation with `std::mem::size_of_val(words)` to satisfy Clippy's `manual_slice_size_calculation`

Although one touched file is the Task 92 counter surface, this packet is filed under Task 91 because it is PR-wide CI cleanup for the current Task 91/92 rollout branch and includes the Task 91 DiskANN helper fix.

## Code Under Review

- Code commit: `2b15523f8301147b8862688de4f14d546a340ca4`
- Files:
  - `src/am/common/candidate_batch.rs`
  - `src/am/ec_diskann/quantizer.rs`

## Validation

Artifacts are under `reviews/task-91/017-rust-checks-clippy-cleanup/artifacts/`.

- `cargo fmt`
  - Result: passed, with existing stable-rustfmt warnings about nightly-only import grouping settings
  - Log: `artifacts/cargo-fmt.log`
- `cargo clippy --all-targets --no-default-features --features pg18,bench -- -D warnings`
  - Result: passed
  - Log: `artifacts/rust-checks-clippy.log`
- `git diff --check`
  - Result: passed
  - Log: `artifacts/git-diff-check.log`

## Review Focus

- Confirm that the `OnceLock` replacement preserves block-kernel counter semantics while satisfying MSRV 1.75 Clippy.
- Confirm that the DiskANN capacity cleanup is behavior-neutral.
