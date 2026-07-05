# SPIRE Vacuum Compaction Leaf PID Guard

## Checkpoint

- Code commit: `a81df13b`
  (`Guard SPIRE vacuum compaction leaf pids`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Review follow-up hardening for vacuum compaction integrity checks

## Summary

This checkpoint addresses the vacuum review note about implicitly trusting the
leaf object header PID during compaction:

- Added an explicit guard that verifies the manifest PID matches the decoded
  leaf object header PID before an affected base leaf is rewritten.
- Switched the affected-leaf rewrite branch to key off the manifest PID, so a
  malformed header PID reaches the new mismatch check instead of bypassing it.
- Added unit coverage for both the accepted PID match and malformed mismatch
  error message.
- Updated the Task 30 plan to record the malformed leaf-header guard.

This is a defensive integrity check only. It does not change normal vacuum
cleanup semantics, delta compaction output, V2 leaf layout, object placement,
or SQL-visible vacuum statistics.

## Changed Files

- `src/am/ec_spire/vacuum.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib compaction_leaf_pid_match --no-default-features --features pg18`
  - `2 passed; 0 failed; 0 ignored; 0 measured; 1110 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `231 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean before code commit

## Notes

- No measurement artifacts are included because this packet does not make a
  measurement claim.
- Broader vacuum compaction I/O-pass reduction remains a separate optimization
  follow-up; this packet only hardens malformed-state handling.
