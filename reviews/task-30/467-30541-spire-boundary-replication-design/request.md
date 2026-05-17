# Review Request: SPIRE Boundary Replication Design

- Branch: `main`
- Code commit: `468bc8e3` (`Design SPIRE boundary replication`)
- Task: Task 30 SPIRE IVF foundation, Phase 5 boundary replication
- Scope: design checkpoint before boundary-replication runtime changes

## Summary

This checkpoint starts Phase 5 with a durable design note:

- adds `plan/design/spire-boundary-replication.md`;
- defines a conservative default-off reloption surface,
  `boundary_replica_count`;
- chooses top-N nearby leaves using the existing route ordering as the first
  boundary predicate;
- caps assignment fanout by reloption and keeps a later margin predicate
  deferred until measurement needs it;
- states that replicated indexes must switch scans to
  `VecIdDedupeEnabled`, while primary-only indexes keep
  `NoReplicaDedupeDisabled`;
- keeps Phase 4 local placement unchanged: replica rows live in their replica
  leaf PID and store placement remains hash-by-PID;
- defines the first diagnostics and measurement gate for recall/storage
  comparison.

No runtime behavior changes in this checkpoint.

## Files

- `plan/design/spire-boundary-replication.md`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

1. Confirm the default-off `boundary_replica_count` reloption is the right
   first control surface, with no session GUC for durable build fanout.
2. Confirm top-N nearby leaves is acceptable as the first predicate before
   adding a margin-based rule.
3. Confirm scan dedupe should be derived from active replica-capable metadata
   rather than a user/session switch.
4. Confirm Phase 4 local placement should remain PID-based without a
   replica-specific store planner.
5. Confirm the diagnostics are sufficient for the first recall/storage packet.

## Validation

Tests not run; this is a design-only checkpoint.
