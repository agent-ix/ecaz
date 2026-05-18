# Review Request: SPIRE Routing Diagnostic Drift Guard

Code checkpoint: `261cf408` (`Guard SPIRE routing diagnostics against drift`)

## Scope

- Extends the recursive routing diagnostics fixture so it compares diagnostic
  selected and deduped route counts against the production recursive route set.
- Keeps the shared traversal/helper refactor open in Phase 10, but marks the
  property-test drift guard complete in
  `plan/tasks/task30-phase10-spire-execution-performance.md`.
- Leaves routing behavior unchanged.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test --no-default-features --features pg18 collect_scan_routing_diagnostics_reports_recursive_levels_and_truncation --lib`

## Review Focus

- Confirm the fixture meaningfully catches production-vs-diagnostic recursive
  routing drift for selected and deduped route counts.
- Confirm deferring the shared traversal/helper refactor remains acceptable for
  Phase 10 while the property guard is in place.
