# Review Request: SPIRE Recursive Nprobe Docs

Head SHA: `e33ad936`

## Summary

The operator diagnostics guide now documents the externally visible recursive
nprobe policy surface:

- `effective_nprobe_per_level`
- `nprobe_policy_per_level`

It states that single-level indexes report one `single_level` entry, while
recursive indexes report one entry per active routing level, ordered from level
1 upward. It also calls out the Phase 3 conservative policy: relation/session
`nprobe` applies at level 1, and levels above 1 probe one child until durable
per-level nprobe configuration lands.

The Task 30 Phase 3 status text now reflects the same distinction: durable
per-level configuration is still deferred, but the effective policy is visible
through diagnostics.

## Files

- `docs/SPIRE_DIAGNOSTICS.md`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `git diff --check`
- Code tests were not run; this checkpoint is documentation-only.

## Review Focus

- Confirm the docs accurately describe the exposed options snapshot arrays.
- Confirm the task-plan text no longer implies the observable per-level surface
  is deferred.
- Confirm the conservative Phase 3 policy warning is clear enough for operators.
