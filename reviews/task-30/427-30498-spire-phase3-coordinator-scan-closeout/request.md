# Review Request: SPIRE Phase 3 Coordinator and Scan Closeout

Head SHA: `9a00540d`

## Summary

The Phase 3 task plan now marks the hierarchy metadata, recursive build
coordinator, centroid materialization, level-local scan primitive, and review
packet checklist items complete.

This is a plan/status checkpoint only. The implementation evidence is in the
preceding Phase 3 code and closeout follow-up packets:

- recursive hierarchy design and shape validation;
- recursive relation build composition and `recursive_fanout` activation;
- recursive scan preload, route descent, leaf validation, and quantized
  candidate collection;
- relation-backed flat/recursive SQL comparison with multiple query vectors and
  top-k set checks;
- routing centroid and recursive options diagnostics;
- per-level nprobe policy arrays on `ec_spire_index_options_snapshot`;
- parse-time `recursive_fanout = 1` rejection;
- degraded recursive routing coverage and three-routing-level descent coverage;
- recursive maintenance split/merge guard until recursive update propagation
  lands;
- recursive draft invariant helper and validation-barrier comments.

The broader hierarchy metadata checklist item is complete for Phase 3 because
active hierarchy shape, level parameters, effective nprobe policy, target
fanout diagnostics, and recursive support flags are now observable. The task
file carries the remaining post-Phase-3 hierarchy metadata follow-ups
explicitly:

- durable per-level `nprobe` storage/configuration;
- durable per-level parameter storage instead of diagnostic reconstruction;
- explicit user-facing per-level fanout configuration beyond current
  diagnostic `target_fanout` exposure.

The task file also carries forward the full closeout follow-up list with
completed items marked `[x]` and deferred items left open, so Phase 4 does not
lose the boundary assumptions.

## Files

- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `git diff --check`

Tests were not rerun for this plan-only checkpoint. The implementation and
closeout commits cited above each carry their focused PG18, Rust, or docs-only
validation.

## Review Focus

- Confirm the hierarchy metadata item is honestly complete for Phase 3 with the
  durable storage/configuration follow-ups explicitly deferred.
- Confirm the carried-forward follow-up list is complete enough to close Phase
  3 without losing Phase 4 prerequisites.
- Confirm the 30498 packet no longer contradicts the task checklist.
