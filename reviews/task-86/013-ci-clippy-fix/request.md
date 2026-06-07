# Task 86 Packet 013: CI Clippy Fix

## Summary

This packet fixes the Task 86 TQ+ clippy failure reported by GitHub CI after packet 012.

The code change is intentionally narrow:

- `src/quant/prod.rs`: convert the two TQ+ range loops over `rotated` into iterator/enumerate loops.
- Preserve the same calibration indexing, centroid lookup, renorm calculation, LUT fill, and bias calculation.

This is a lint-only cleanup for the existing TQ+ implementation. It does not change the packet 011 benchmark suite, benchmark interpretation, storage format plan, or TQ+ measurement conclusions.

## Validation

Artifact manifest:

- `reviews/task-86/013-ci-clippy-fix/artifacts/manifest.md`

Validation log:

- `reviews/task-86/013-ci-clippy-fix/artifacts/cargo-clippy-pg18-bench.log`

Command:

```sh
cargo clippy --all-targets --no-default-features --features pg18,bench -- -D warnings
```

Result:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 10m 44s
```

## CI Context

The PR's Rust/clippy jobs failed on `clippy::needless_range_loop` in the two TQ+ loops fixed here.

The PR's separate `Test Quality Coverage` job also failed in `hardening/careful/src/spire.rs`. This branch has no diff against `origin/main` for that file, so this packet treats it as outside Task 86's TQ+ scope unless the reviewer explicitly asks for unrelated hardening/careful cleanup in this PR.
