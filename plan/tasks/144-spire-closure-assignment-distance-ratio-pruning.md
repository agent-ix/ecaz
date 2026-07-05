# Task 144: SPIRE Closure Assignment + Query-Time Distance-Ratio Pruning (SPANN Triad)

Status: proposed (2026-07-04; remediation program task 4 of 6). Subsumes
Task 140 (adaptive termination) — distance-ratio pruning is the stronger
form of the same idea and does not require a recall-SLO contract change for
its sound configuration study.
Owner: coder (to be assigned). One coder, one branch.
Priority: P1 — the architectural bet; the path to a 1–4-probe operating
regime. Depends on Tasks 141 (substrate) and 143 (ranking fix first, so this
measures assignment/pruning, not the ranking defect).

## Why

SPANN (Chen et al., NeurIPS 2021 — the reference architecture ADR-049 builds
toward) reaches high recall probing only 1–4 posting lists via three coupled
mechanisms. SPIRE implements none of them:

1. **ε-bounded closure assignment**: SPANN replicates a vector into every
   cluster whose centroid distance is within a ratio bound of the nearest —
   replication adapts per-vector to boundary ambiguity. SPIRE uses a fixed
   `boundary_replica_count` 0–8 (top-N+1 nearest centroids,
   `src/am/ec_spire/build/routing_plan.rs:122-150`; default 0). "closure"
   appears nowhere in the codebase.
2. **Query-time centroid-distance-ratio pruning**: SPANN probes only lists
   with `dist ≤ (1+ε)·dist_best`, making heavy replication cheap at query
   time. Entirely absent — our nprobe is a fixed count. This is why the Task
   139 grid found every b1/b2 cell off the Pareto frontier: we pay SPANN's
   replication cost (inflated surface, duplicate handling) without its
   pruning payoff.
3. **Balanced posting-list sizes**: hierarchical k-means exists but no
   size-balancing constraint; leaf-size variance is unmeasured.

Prerequisites now exist: honest `distinct_recall@k` (Task 138) and working
cross-node identity dedupe (Task 137/ADR-083) — replication recall claims are
no longer duplicate-inflated. Dormant scaffolding: `ec_spire.adaptive_nprobe`
score-gap halving (`scan/routing.rs:735-810`, off, measured once at noise);
`nprobe_per_level` (never benched).

## Goal

distinct_recall@10 ≥ 0.99 while probing a corpus fraction comparable to
textbook IVF (1–5% of row-instances), by replacing fixed-count replication
and fixed-count probing with ratio-bounded versions of both.

## Scope

### Phase 0 — Geometry diagnostics

- Measure leaf-size variance and per-query true-neighbor list concentration
  (how many lists hold the true top-10 under closure vs single assignment)
  at 50k/100k. This bounds the achievable probe count before building
  anything.

### Phase 1 — ε-closure assignment (build side)

- Reloption `closure_epsilon`: assign each vector to all centroids with
  `dist ≤ (1+ε)·dist_nearest` (cap per-vector replicas; report replication
  factor and storage delta). Keep `boundary_replica_count` as the fixed-count
  fallback for A/B.

### Phase 2 — Distance-ratio probe pruning (query side)

- GUC `probe_distance_ratio`: after route ranking, keep only leaves within
  the ratio of the best route (floor of min-probes, ceiling of nprobe).
  Compose with Task 143's overfetch. Per-query probed-count distribution in
  the metrics.

### Phase 3 — Matrix + decision

- A/B grid at 10k/50k/100k: {single-assignment, fixed-b, closure-ε} ×
  {fixed nprobe, ratio pruning}, distinct recall + % scanned + p50 + storage.
  Success: ≥0.99 distinct recall at ≤5% row-instances scanned at 50k/100k.
  If closure+pruning cannot reach it, escalate per ADR-051/060 reopen
  conditions (multi-probe / anisotropic centroid scoring) with the funnel
  evidence attached.

## Required Evidence

- `ecaz bench suite`, release build, 10k/50k/100k A/B per the closeout rule;
  storage step mandatory (replication factor is a storage trade);
  per-query probed-list and recall distributions (tail, not just mean).

## Non-Goals

- No unsound early termination revival (Task 131's shelved shape).
- No learned routing (ADR-052/053) in this task — escalation path only.

## Acceptance Criteria

1. Closure assignment + ratio pruning landed behind reloption/GUC, default
   off.
2. Phase 0 concentration diagnostic published (the "is 1–4 probes even
   geometrically possible here" number).
3. Full matrix with pre-registered success criteria; promote/iterate/escalate
   decision with numbers.

## References

- `spec/adr/ADR-049-spire-on-single-level-ivf-foundation.md` (SPANN summary)
- `src/am/ec_spire/build/routing_plan.rs:122-150`, `scan/routing.rs:735-810`,
  `options/mod.rs:383-472,1571`
- `plan/tasks/140-spire-adaptive-termination-recall-slo.md` (subsumed)
- `plan/tasks/137-*.md`, `138-*.md` (prerequisites: honest dedupe + metric)
- `spec/adr/ADR-051`, `ADR-060`, `ADR-061`, `ADR-062` (reopen conditions met;
  escalation paths)
