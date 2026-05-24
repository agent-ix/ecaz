# Task 50 Review Request: DiskANN Vacuum Helper Guards

## Summary

This slice narrows DiskANN routine unsafe propagation around vacuum and repair helpers.

Several helpers already owned their raw PostgreSQL callback or relation contracts internally, so their signatures are now safe:

- `maybe_apply_vacuum_rewrite_test_injection`
- `indexed_ecvector_attnum`
- `resolve_vacuum_heap_relation`
- `fill_vacuum_neighbor_slots`
- `plan_vacuum_fill_candidates_for_target`
- `callback_marks_heap_tid_dead`

The remaining unsafe operations stay inside documented local boundaries: callback invocation, heap slot allocation, index metadata reads, and page rewrites.

## Unsafe Burn Down

- `src/am/ec_diskann/routine.rs` unsafe token count: `70 -> 57`
- `src/am/ec_diskann/routine.rs` direct `unsafe { ... }` blocks: `42 -> 36`
- `src/` total unsafe token count after this slice: `2594`

## Code Commit

- `789d118102aff39f2a54849aaa7373b9a8ba7f96` Tighten DiskANN vacuum helper guards

## Validation

- `rustfmt --edition 2021 --check src/am/ec_diskann/routine.rs`
  - log: `artifacts/rustfmt-diskann-routine.log`
  - passed with existing stable-rustfmt warnings for unstable import grouping options
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - log: `artifacts/cargo-check-pg18-bench.log`
  - passed with existing unused-import warning in `src/am/mod.rs`
- `cargo test --lib ec_diskann --no-default-features --features pg18,pg_test --no-run`
  - log: `artifacts/cargo-test-ec-diskann-no-run.log`
  - passed with existing Hadamard test-helper dead-code warnings
- `git diff --check`
  - log: `artifacts/git-diff-check.log`
  - passed
- DiskANN routine unsafe scan:
  - log: `artifacts/diskann-routine-unsafe-scan.log`

