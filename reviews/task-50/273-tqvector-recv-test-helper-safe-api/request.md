# Task 50 Review Request: Tqvector Recv Test Helper Safe API

## Summary

This slice removes the remaining unsafe markers from `src/tests/hnsw_misc.rs`.
The malformed binary receive fixtures now call `recv_via_string_info` as a safe
local helper; the helper constructs the fixture `StringInfoData` and calls the
safe `tqvector_recv(Internal::new(raw))` path directly.

`cargo check` initially caught that the last attempted `Internal::new` unsafe
block was unnecessary, so this packet removes it too. The file now has no
`unsafe` matches.

## Unsafe Burndown

- Previous broad count from packet 272: `2208`
- Current broad count: `2205`
- Net: `-3`

## Validation

Artifacts are under `reviews/task-50/273-tqvector-recv-test-helper-safe-api/artifacts/`.

- `git-diff-check.log`: passed
- `rustfmt-check.log`: standalone rustfmt skipped because changed file is a module-included test source; syntax/format viability was checked by cargo parsing
- `hnsw-misc-unsafe-grep.log`: no `unsafe` matches remain in `src/tests/hnsw_misc.rs`
- `unsafe-count.log`: `2205`
- `cargo-check-pg18-bench.log`: passed with the existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`
- `cargo-test-lib-pg18-pgtest-no-run.log`: passed with existing Hadamard test-only dead-code warnings
