# Task 226: ec_distann Current-Head BW8 Transfer Screen

Status: **proposed** (2026-08-21). Priority: P0 recall/latency.

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

## Scope

- Same-generation 100k screen with paired per-query recall and prediction
  identity accounting.
- Report rounds, expanded nodes, frontier insertions, owner requests,
  request/response bytes, transport wait, materialization work, and tails.
- If useful, confirm at 10k/50k/100k using release `ecaz bench suite`.
- Record a Pareto point without changing the default if recall rises at a
  material latency cost.

## Non-goals

- Re-running BW64/H8, cap-16,384 heads, or Task 206's top-k-200/L200 surface.
- Adaptive search or graph rebuilds; Task 227 owns those.
- Stacking a materialization candidate in the BW8 arm.

## Acceptance

1. One-generation A/A and A/B provenance proves that beam width is the only
   moving behavior.
2. The 100k screen reports paired recall and complete work/latency attribution.
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
