# Review Request: SPIRE Advisory Lock Namespace

Code checkpoint: `c9afb529` (`Document SPIRE governance lock namespace`)

## Summary

This documentation slice addresses reviewer P2 from
`30717-spire-libpq-global-dispatch-governance`: publish the PostgreSQL
advisory-lock class/object ranges used by SPIRE remote-search dispatch
governance before async/pipeline executor work lands.

## Scope

- Adds an `Advisory Lock Namespace` section to
  `plan/design/spire-libpq-executor-budget.md`.
- Records the global dispatch slot range:
  `class_id = 730000000 + slot`, `object_id = 0`, for slots `0..4095`.
- Records the per-node dispatch slot range:
  `class_id = 731000000 + slot`, `object_id = bit_preserving_i32(node_id)`,
  for slots `0..4095`.
- Notes that other extension features, operator scripts, and runbooks must not
  use these class ranges.
- Adds the same reserved-range contract to ADR-058.
- Updates the Phase 11 task file to mark the namespace documentation closed.
- Includes the `pg_locks` inspection hint requested as reviewer P3 context.

## Validation

- `git diff --check`
  - exited `0`

No runtime tests were run; this is a documentation-only contract checkpoint.

## Review Questions

- Are the documented class ranges and object-id mapping precise enough for
  operator/runbook use?
- Is ADR-058 the right durable home for this namespace, with the detailed range
  table remaining in the budget design doc?
