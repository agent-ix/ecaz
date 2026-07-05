# SPIRE Object Tuple Scan Contract

## Checkpoint

- Code commit: `225c2055`
  (`Document SPIRE object tuple scan contract`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Round review follow-up for relation object tuple scans

## Summary

This checkpoint documents the callback contract for
`scan_object_tuples`.

The visitor is invoked while the current object page is held under
`BUFFER_LOCK_SHARE`. The code now states that visitors should be limited to
CPU-only tuple inspection and copying bytes into caller-owned state, and should
not read or pin other pages in the same relation from inside the callback.

## Changed Files

- `src/am/ec_spire/page.rs`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `git diff --check`

Tests were not run because this is a comment-only safety documentation change.

## Notes

- This responds to the round-review warning that future diagnostics could
  accidentally perform I/O from inside a buffer-locked visitor.
