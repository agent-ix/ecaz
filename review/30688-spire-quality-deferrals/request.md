# Review Request: SPIRE Phase 9.7 Quality Deferrals

Checkpoint: `69667247` (`Defer remaining SPIRE quality experiments`)

## Scope

- Adds ADR-060 for anisotropic centroid scoring deferral.
- Adds ADR-061 for IMI reshape deferral.
- Adds ADR-062 for query difficulty estimator deferral.
- Updates the ADR index with all three new deferred decisions.
- Updates the detailed Phase 9 task file so every Phase 9.7 item is now either
  implemented or explicitly ADR-deferred.
- Updates the main Task 30 overview so the Phase 9 quality experiment pointer
  matches the detailed Phase 9 file.

## Rationale

The local real10k baseline in `review/30686-spire-phase9-quality-baseline`
shows recall@10 saturation by `nprobe=16`, and the adaptive `nprobe` treatment
in `review/30687-spire-adaptive-nprobe` is the only Phase 9.7 item classified
as must-land locally. The remaining items need harder local fixtures,
hard-query subsets, or research-track evidence before implementation can prove
value.

## Validation

- `git diff --check 69667247^..69667247`

## Review Focus

- Confirm the deferral ADRs meet the external Phase 9 closeout requirement:
  each item has a baseline reference, rationale, and concrete revisit
  conditions.
- Confirm the task checkboxes now match reality and do not imply product-scale
  performance claims.
