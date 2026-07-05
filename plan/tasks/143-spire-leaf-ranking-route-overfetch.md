# Task 143: SPIRE Leaf Ranking Fix + Route Overfetch

Status: proposed (2026-07-04; remediation program task 3 of 6).
Owner: coder (to be assigned). One coder, one branch.
Priority: P1 — small change, high signal; directly tests the standout
routing defect. Depends on Task 141 (substrate); independent of 142.

## Why

SPIRE ranks leaves by an additive hierarchical score
`total_score() = path_score + score` (`src/am/ec_spire/scan/types.rs:111-112`;
parent path score folded in at `scan/routing.rs:517,1179,1359-1366`), while
standalone ec_ivf ranks by pure `IP(query, leaf_centroid)`
(`src/am/ec_ivf/scan.rs:1228`). At the grid config the hierarchy is trivial
(2 levels, all 1024 leaf centroids scored exactly in f32), so the parent term
is injected noise on an otherwise-exact leaf score: it can push a truly-near
leaf out of the top-nprobe when its parent centroid is less aligned. There is
no cushion: `max_leaf_routes = beam_width = effective_nprobe` with no
overfetch multiplier (`options/mod.rs:1604-1616`, `:1647`), so a true leaf
ranked at nprobe+1 is silently dropped. Task 121 already localized ALL recall
loss to route/leaf selection (route-stage containment == final recall), and
Task 75 measured SPIRE scoring 15.5M candidates vs IVF ~75k at matched recall
(37×).

## Goal

Leaf selection at least as precise as standalone IVF's at equal nprobe:
rank by pure leaf IP, cushion residual ranking error with route overfetch,
and quantify the containment lift.

## Scope

### Phase 0 — Leaf-only ranking A/B

- Rank leaf candidates by leaf `score` alone; keep `path_score` for
  tie-breaks/descent only. Gate with a default-off GUC for A/B.
- Re-run the Task 121 route-containment funnel at fixed nprobe ladder,
  50k/100k, distinct_recall@10, corpus-fraction scanned.

### Phase 1 — Route overfetch

- Select `α·nprobe` leaf routes by route score, exact-rerank by leaf IP,
  keep top nprobe (α sweep {1.25, 1.5, 2}). Measure containment lift vs
  routing-cost delta.

### Phase 2 — Decision

- If containment at nprobe X after fixes ≥ containment at nprobe 2X before,
  promote as default and re-anchor the frontier point for Task 146.
  Otherwise publish the negative with funnel evidence and hand the
  precision question to Task 144.

## Required Evidence

- Route-containment funnel tables (per-nprobe: containment, distinct recall,
  % scanned, p50) A/B at 10k/50k/100k per the closeout rule, release build,
  `ecaz bench suite`.

## Non-Goals

- No assignment/replication changes (Task 144). No rerank changes (145).

## Acceptance Criteria

1. Leaf-only ranking + overfetch landed behind GUCs, A/B'd at 10k/50k/100k.
2. Containment funnel published; promote/iterate/negative decision with
   numbers.

## References

- `src/am/ec_spire/scan/types.rs:111-112`, `scan/routing.rs:503-519,1173-1187`,
  `options/mod.rs:1604-1647`, `src/am/ec_ivf/scan.rs:1199-1233,1347-1408`
- `plan/tasks/121-spire-coarse-routing-recall-doe.md` (containment funnel)
- `plan/tasks/75-*.md` (37× candidate-surface gap vs IVF)
