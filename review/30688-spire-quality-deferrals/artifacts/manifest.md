# SPIRE Quality Deferrals Artifact Manifest

Head SHA: `6966724741f7bd2106c09e783f6fb67a20be55c3`
Packet/topic: `30688-spire-quality-deferrals`
Timestamp: `2026-05-09T16:10:00-07:00`

This packet records the ADR-backed disposition for the remaining Task 30 Phase
9.7 quality experiments after the canonical baseline
`review/30686-spire-phase9-quality-baseline` and adaptive treatment
`review/30687-spire-adaptive-nprobe`.

## Artifacts

| Artifact | Lane | Command | Key result lines |
| --- | --- | --- | --- |
| `git-diff-check.log` | docs validation | `git diff --check 69667247^..69667247` | Exit 0; no whitespace errors. |

## Decisions

| Item | Disposition | Evidence |
| --- | --- | --- |
| Anisotropic centroid scoring | ADR-deferred by `spec/adr/ADR-060-spire-anisotropic-centroid-scoring-deferred.md` | Baseline packet 30686 shows real10k recall@10 saturates to `1.0000` by `nprobe=16`, leaving no useful recall headroom for this treatment. |
| IMI reshape | ADR-deferred by `spec/adr/ADR-061-spire-imi-reshape-deferred.md` | Baseline packet 30686 records single-IVF storage at real10k scale; IMI needs a larger local fixture to exercise storage/routing tradeoffs. |
| Query difficulty estimator | ADR-deferred by `spec/adr/ADR-062-spire-query-difficulty-estimator-deferred.md` | Adaptive packet 30687 provides deterministic adaptive `nprobe` diagnostics; learned/difficulty estimators remain research track under ADR-052 and ADR-053 until diagnostics show a concrete gap. |

## Task Updates

- `plan/tasks/task30-phase9-spire-graph-architecture.md` now has all four
  Phase 9.7 items checked, either implemented or ADR-deferred.
- `plan/tasks/30-spire-ivf-foundation.md` now marks the Phase 9 quality
  experiment overview complete and points to the detailed Phase 9 task file.
- `spec/adr/index.md` lists ADR-060, ADR-061, and ADR-062.
