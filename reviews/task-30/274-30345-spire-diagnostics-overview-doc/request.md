# SPIRE Diagnostics Overview Doc

## Checkpoint

- Code commit: `440714f3`
  (`Document SPIRE diagnostic SQL surface`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Round review follow-up for the diagnostic SQL surface

## Summary

This checkpoint adds `docs/SPIRE_DIAGNOSTICS.md` as the operator-facing map of
SPIRE diagnostic SQL functions.

The doc records:

- which diagnostic functions are the recommended starting points
- the full current function map for the `ec_spire_index_*` diagnostic surface
- operator vs. debug audience labels
- notes on empty-index row shapes, strict local single-store behavior, and why
  repeated per-row scan/epoch columns are intentional

The Task 30 plan now points to this doc instead of saying deeper operator
guidance remains open.

## Changed Files

- `docs/SPIRE_DIAGNOSTICS.md`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `git diff --check`

Tests were not run because this is a documentation-only review follow-up.

## Notes

- This responds to the round-review recommendation to add a single source of
  truth for the growing SPIRE diagnostic SQL surface.
