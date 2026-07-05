# SPIRE Vacuum Compaction Object-Version Guard

## Checkpoint

- Code commit: `5281b248`
  (`Guard SPIRE compaction object versions`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Round review follow-up for packet `30320`

## Summary

This checkpoint adds the object-version counterpart to the existing vacuum
compaction PID guard.

When compaction rewrites an affected base leaf, it now verifies that the active
object manifest entry and the decoded object header agree on
`object_version`. A mismatch now errors before compaction can publish a
replacement leaf based on inconsistent manifest/header metadata.

The rewrite version is derived from the validated matched version, then
incremented for the replacement V2 base leaf.

## Changed Files

- `src/am/ec_spire/vacuum.rs`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test compaction_leaf_object_version_match --no-default-features --features pg18`
  - `2 passed; 0 failed; 1121 filtered out`
- `cargo test --lib ec_spire_vacuum --no-default-features --features pg18 -- --nocapture`
  - `4 passed; 0 failed; 1119 filtered out`
- `git diff --check`

## Notes

- This follows the round-review recommendation to mirror the existing
  `require_compaction_leaf_pid_match` guard for `object_version`.
