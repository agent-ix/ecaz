# Review Request: Task 28 IVF Result Emission

Scope: Phase 4 result-emission checkpoint. `ec_ivf` `amgettuple` now drains
materialized probe candidates, sets heap TIDs, publishes ORDER BY scores, and
clears score output on exhaustion.

Task: `plan/tasks/28-ivf-access-method.md` Phase 4

Branch: `task28-ivf`

Head SHA: `4508fc4606b029bf47fb1e23c3e59c48b6870122`

Owner: coder2

Files:

- `src/am/ec_ivf/scan.rs`
- `src/am/ec_ivf/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo check --no-default-features --features pg18 --tests`
- `git diff --check`

Validation notes:

- Validation was PG18-only per the current AGENTS policy.
- The new PG test was compiled but not run. No test suite was executed for
  this checkpoint.
- No measurement claim is made in this packet.

## Summary

This slice makes populated IVF scans produce ordered scan outputs:

- `amgettuple` now drains score-ordered candidates from scan opaque state.
- Each produced candidate sets `xs_heaptid` and `xs_orderbyvals[0]`.
- Exhaustion and empty-index paths clear the ORDER BY null flag and return
  `false`.
- Backward scan rejection and amrescan-required gating remain intact.
- Adds a PG debug helper and test coverage that verifies emitted heap TIDs are
  unique, scores are finite, outputs are score ordered, and exhaustion clears
  the score slot.

## Review Focus

Please review for:

- Whether result emission should land before bounded top-k, given the current
  all-candidates materialization path.
- Whether ORDER BY score ownership and clearing mirrors the HNSW scan contract
  closely enough.
- Whether the direct debug helper is adequate, or whether this needs a
  heap-backed `index_getnext_tid` helper before broader PG tests.
- Whether the remaining posting-list checklist item should close only after
  bounded top-k state replaces all-candidates sorting.

## Non-Goals

This packet does not implement bounded top-k state, storage-format-specific
scoring, rerank mode, live insert, vacuum, planner costing, or any measurement
claim.
