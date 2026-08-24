# Task 226: ec_distann Current-Head BW8 Transfer Screen

Status: **implementation/evidence complete; packets 002/003 review-open;
USEFUL CANDIDATE — POLICY REVIEW; default unchanged** (2026-08-24). The
registered gate passes at 10k, 50k, and 100k, but 50k/100k p99 regressions are
carried to outside policy review. Evidence:
`reviews/task-226/002-current-head-100k/` and
`reviews/task-226/003-full-scale-decision/`. Priority: P0 recall/latency.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`, candidate
TRAV-31. This is a changed-premise transfer test, not a rerun of Task 215's
BW64/H8 arm or Task 194's BW8/H50 arm.

## Why

Task 188's accepted research surface found BW8/H100 simultaneously improved
recall and latency over BW4/H100 at 10k/50k/100k; at 100k it moved
0.9740/28.8 ms to 0.9805/26.5 ms. That run used a non-production 16,384
training-landmark head and predated the current sharded membership head,
Algorithm-1 pushdown, gateway copies, and final post-221 materialization path.

Task 215 tested BW64/H8 on the shipped surface and found a higher-recall,
higher-latency trade. It did not test whether the smaller BW8/H100 win transfers
to current production.

## Goal

Run one current-production, same-generation BW4/H100 versus BW8/H100 A/B and
decide whether BW8 is a genuine Pareto improvement, a recall/latency trade, or
a non-transferable historical result.

## Entry gate

1. Use the conforming sharded owner path, fixed 4,096 head default, current
   pushdown/gateway behavior, lazy-10, and current materialization defaults.
2. Hold H100, L32, head settings, query set, generation, storage, and all other
   behavior fixed; only beam width changes.
3. Capture feature-only stage/work counters without using instrumented latency
   as the sole production decision row.
4. Run the production-latency and full-attribution surfaces on separate fresh
   fixtures. Within each surface, all variants share one immutable generation;
   do not reuse a fixture after its lifecycle drills.

## Scope

- Same-generation 100k production screen with four ordered runtime variants:
  `aa-control` BW4, `aa-candidate` BW4, `bw4-control` BW4, and
  `bw8-candidate` BW8. Require byte-identical A/A predictions and compute the
  registered paired per-query recall comparison from the exact
  `bw4-control`/`bw8-candidate` names. All other variant fields are identical.
- Report rounds, expanded nodes, frontier insertions, owner requests,
  request/response bytes, transport wait, materialization work, and tails.
- Capture the same BW4/BW8 arms on a separate full-metrics 100k fixture for
  attribution; its instrumented latency is diagnostic, not the decision row.
- **Pre-registered 100k gate:** ADVANCE to 10k/50k/100k only when BW8 is not
  worse on paired recall (point delta >= 0 and paired-bootstrap lower bound >=
  0) and either (a) warm mean improves by at least 1.0 ms or 5% with no >5%
  p95/p99 regression, or (b) paired recall improves with a lower bound >= 0
  while warm mean and p95 regress by no more than 5%. A recall gain coupled to
  a >5% warm-mean or p95 regression is classified as a recall/latency trade
  and STOPs under Task 219's interactive-default policy. Any result satisfying
  neither advance arm is non-transferable and STOPs. Storage/topology must
  remain conforming in every case.
- If the gate advances, confirm at 10k/50k/100k using release
  `ecaz bench suite` and apply the same rule at every scale.
- Record a Pareto point without changing the default if recall rises at a
  material latency cost.

## Non-goals

- Re-running BW64/H8, cap-16,384 heads, or Task 206's top-k-200/L200 surface.
- Adaptive search or graph rebuilds; Task 227 owns those.
- Stacking a materialization candidate in the BW8 arm.

## Acceptance

1. One-generation A/A and A/B provenance proves that beam width is the only
   moving behavior.
2. The 100k screen reports byte-identical A/A, paired BW4/BW8 recall, a clean
   production latency row, and complete separate-fixture work attribution.
3. A useful candidate receives 10k/50k/100k recall/latency/storage evidence;
   otherwise STOP.
4. No default changes without a reviewed production-policy disposition.

## Required review packets

1. `reviews/task-226/001-plan/`
2. `reviews/task-226/002-current-head-100k/`
3. `reviews/task-226/003-full-scale-decision/` (only after a useful screen)

## References

- `reviews/task-188/005-batch10-reconfirmation/`
- `reviews/task-215/003-release-matrix-and-decision/`
- Task 219 default-policy decision
