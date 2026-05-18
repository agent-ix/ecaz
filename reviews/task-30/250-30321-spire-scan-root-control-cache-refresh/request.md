# SPIRE Scan Root-Control Cache Refresh

## Checkpoint

- Code commit: `576e8d11`
  (`Refresh SPIRE scan root control cache`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Review follow-up hardening for scan descriptor root-control caching

## Summary

This checkpoint addresses the architecture-review concern around stale
root-control state on repeated rescans:

- `SpireScanOpaque` still caches root control across rescans, but each rescan
  now observes the current root/control page and refreshes the cached value
  when `active_epoch` changes.
- Same-epoch observations continue to reuse the cached state, preserving the
  invariant that a given active epoch has stable manifest locators.
- Added unit coverage for initial cache population, same-epoch reuse, and
  refresh on a newer active epoch.
- Updated the Task 30 plan to record the cache refresh behavior.

This does not change scan candidate ranking, routing, diagnostics, or
publication semantics. It only prevents a long-lived scan descriptor from
continuing to use stale root-control state after a later rescan sees a new
active epoch.

## Changed Files

- `src/am/ec_spire/scan.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib scan_opaque_refreshes_root_control_when_active_epoch_changes --no-default-features --features pg18`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1112 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `232 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean before code commit

## Notes

- No measurement artifacts are included because this packet does not make a
  measurement claim.
- Heavier snapshot/manifest caching remains future work; this checkpoint only
  fixes the active-epoch invalidation boundary.
