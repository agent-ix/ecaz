# Review Request: SPIRE Routing Diagnostic Depth Guard

Code checkpoint: `43a26989` (`Extend SPIRE routing diagnostic drift guard`)

## Scope

- Completes the Phase 10.1a depth > 2 routing diagnostic drift guard item.
- Adds a three-level routing hierarchy fixture: root level 3, intermediate
  level 2, parent level 1, and leaf routes.
- Compares diagnostic deduped route counts and selected child counts against
  the production recursive route output at root, intermediate, and leaf levels.
- Marks the Phase 10.1a depth guard item complete.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test --no-default-features --features pg18 collect_scan_routing_diagnostics_matches_production_on_three_level_hierarchy --lib`
- `cargo test --no-default-features --features pg18 collect_scan_routing_diagnostics --lib -- --test-threads=1`

## Notes

- The combined routing-diagnostics filter must run with `--test-threads=1`.
  The parallel Rust test harness can enter pgrx GUC-backed scan-plan code from
  multiple threads and trip pgrx's PostgreSQL FFI thread guard.

## Review Focus

- Confirm the fixture genuinely covers an intermediate recursive level, not
  only root-to-leaf diagnostics.
- Confirm the production-vs-diagnostic comparisons are strong enough to catch
  drift in selected and deduped counts across all recursive levels.
