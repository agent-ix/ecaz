# Task 146: SPIRE Honest Pareto Confirmation (supersedes Task 139 phases 2–4)

Status: proposed (2026-07-04; remediation program task 6 of 6). Supersedes
the unrun phases of Task 139 on the fixed substrate.
Owner: coder (to be assigned). One coder, one branch.
Priority: P1 gate — the program's evidence bar. Depends on 141+142
(substrate + floor), consumes 143/144/145 outcomes.

## Why

Task 139 phase 1 established (on a debug substrate, but recall/scan-fraction
columns are trusted): no nlists×boundary configuration reaches
distinct_recall@10 ≥ 0.999 at ≤15% corpus scanned — the ceiling at ≤15% was
~0.966, grid max 0.9930 at 51% scanned, and every fixed-count replication
cell sat off the frontier. Geometry tuning alone cannot meet the target; the
mechanism fixes (142–145) exist precisely to bend that frontier. This task
re-measures the program's end state honestly and decides
promote/iterate/shelve for the distributed SPIRE research lane.

## Goal

The definitive release-build Pareto frontier for post-remediation SPIRE:
distinct recall vs corpus-fraction scanned vs p50, against release IVF/HNSW
baselines on the same host/corpus.

## Scope

### Phase 0 — Shape selection

- From 143/144/145 packets, pick ≤6 candidate shapes (including one
  fixed-count-replication control and the historical n128/b0 anchor).
- Treat Task 144 packet 012 as a negative gate input: closure/ratio pruning does
  not reach the 1-4-probe / <=5% scan regime at 50k/100k, and the least-bad
  0.99 scan fraction regresses 2.96% -> 35.68% -> 78.66% with corpus size.
- Carry Task 139 Phase 2's router-saturation levers as shape axes if
  143/144 leave recall short of the gate: `top_graph_search_list_size`
  {96, 128, 200, 400} (Task 121 found it clipped at 96 with recall still
  rising) and `training_sample_rows` {50k, 100k, full} (centroid quality).

### Phase 1 — Matrix

- 10k/50k/100k (1m if 100k is promising), 200+ queries, standard sweep,
  `source_identity=include`, block-pruning counters live, per-node build
  profile in the manifest. Both single-instance and 3-worker multinode runs
  on the same shapes (the attribution split Task 139 phase 3 planned but
  never ran).

### Phase 2 — Verdict

- Gate: distinct_recall@10 ≥ 0.999 at ≤10–15% row-instances scanned, p50
  within a documented factor of release IVF at matched recall
  (anchor: IVF 100k 0.9980 @ 37.7 ms).
- Outcomes: promote the winning shape as the SPIRE default; or iterate with
  the ADR-051/060 escalation from Task 144; or shelve the distributed lane
  with the honest curve published.

## Required Evidence

- `ecaz bench suite` only, canonical configs where possible; pre-registered
  criteria before the matrix runs; suite manifests with per-node
  `ecaz_build_profile()`; results.jsonl for every cited number.

## Non-Goals

- No new mechanisms in this task — measurement and decision only.

## Acceptance Criteria

1. Full matrix on release substrate, single- and multi-instance.
2. Frontier table + comparison to IVF/HNSW release baselines.
3. Promote / iterate / shelve decision with pre-registered criteria.

## References

- `plan/tasks/139-spire-routing-selectivity-pareto.md` (superseded phases)
- `reviews/task-139/001-phase1-nlists-boundary-grid/` (phase-1 grid +
  debug-taint feedback 2026-07-04-01-agent-ix.md)
- `benchmarks/task76-intel-local-spire-pareto/manifest.md` (release anchors)
- Tasks 141–145 (inputs)
