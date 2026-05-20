# Task 50 Review Request: Common Parallel Validation Warning Cleanup

## Summary

This packet follows packet 025 with a narrow validation cleanup in `src/am/common/parallel.rs`.

Code commit:

- `a628461b Clean common parallel validation warnings`

The change removes the now-unused `EC_PARALLEL_WORKER_SLOT_CLAIMED` re-export from the production module surface, imports it only in tests, and replaces `std::ptr::from_ref` with Rust 1.75-compatible raw pointer casts.

## Unsafe Count Status

Counts are from `artifacts/block-count-after.log`.

| File | Task 50 start | Follow-up count | Target | Status |
| --- | ---: | ---: | ---: | --- |
| `src/am/common/parallel.rs` | 63 | 38 | <=44 | met |

## Review Notes

- No parallel scan behavior changes are intended.
- The Rust 1.75-compatible pointer casts preserve the packet 025 reference-based helper shape.
- The test-only claimed-slot constant remains available to the test module without keeping an unused production re-export.

## Validation

- `make unsafe-block-count PATHS='src/am/common/parallel.rs'` passed with count 38.
- `rustfmt --edition 2021 --check src/am/common/parallel.rs` passed, with only existing stable-rustfmt warnings about unstable import options.
- `git diff --check` passed.
- `cargo check --all-targets --no-default-features --features pg18,bench` passed, with the existing `src/am/mod.rs` unused export warning.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` failed on the existing repo-wide clippy backlog; the packet-local MSRV and unused re-export findings from `src/am/common/parallel.rs` are cleared.
