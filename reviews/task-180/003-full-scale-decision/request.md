---
task: 180
packet: 003-full-scale-decision
role: coder
status: open
date: 2026-07-15
---

# Review request: full-scale bounded-head confirmation

> Decision rationale corrected on 2026-07-15 by packet 004. The measurements
> and width/seed-tuning NO-GO remain valid, but proposed NFR-017 targets were
> not stakeholder-approved hard task gates.

Please review the completed Phase 2 matrix and Task 180 NO-GO decision.

## Pre-registered selection

Phase 1 selected persisted cap 4096 / width 64 / 64 returned seeds. Cap 16384
had the highest nominal recall (0.9440), but its 0.9330-0.9533 CI overlaps the
cap-4096 width-64 cells' 0.9158-0.9385 CI, while its p50 is worse (45.2 ms) and
its cached head is four times larger. The registered recall/overlapping-CI/p50/
head-byte order therefore selects the cap-4096 cell; within the same-run seed
sweep, 64 seeds had the lowest p50 (40.2 ms) at identical recall 0.9280.

The exact-neighbor trigger did not fire because bounded recall 0.9280 is 0.0690
below the owner oracle's 0.9970, not within 0.0050.

## Registered matrix

At each of 10k/50k/100k, one immutable cap-4096 physical build measures exactly:

1. production persisted head, width 32 / seeds 32;
2. owner-scan oracle, width 32 / seeds 32; and
3. selected bounded persisted head, width 64 / seeds 64.

All arms hold graph degree 32, BW4/H100, RaBitQ neighbor scoring, corpus/query
identity, and topology fixed. The artifact manifest is the packet-local source
of truth.

## Requested review focus

1. Confirm the Phase 1 tie-break was applied literally and the selected cell is
   bounded/non-O(N).
2. Confirm the three arms and 10k/50k/100k matrix match Task 180 Phase 2.
3. Check per-scale topology, provenance, remote engagement, recall/CI, latency,
   build, storage, and head accounting when results land.
4. Report the proposed NFR-017 comparison targets separately from the relative
   production A/B decision.

## Completed result

All three scales passed exact/disjoint topology, release-SHA unanimity, remote
engagement, storage accounting, and suite integrity checks. The selected
bounded arm measured:

| Scale | Distinct recall@10 (95% CI) | Warm p50 / p95 |
| --- | ---: | ---: |
| 10k | 0.9990 (0.9964-0.9997) | 33.50 / 39.10 ms |
| 50k | 0.9540 (0.9439-0.9623) | 43.20 / 53.10 ms |
| 100k | 0.9280 (0.9158-0.9385) | 40.90 / 52.20 ms |

At 100k, unchanged production was 0.9275 recall / 40.30 ms p50, while the
diagnostic O(N) owner oracle was 0.9970 / 2445.20 ms. Width64/seeds64 therefore
does not recover recall and is not a latency improvement.

## Decision for review

The candidate is a **NO-GO for width/seed tuning** because its 100k recall is
statistically flat versus production (0.9280 versus 0.9275) while p50 is 0.6 ms
slower. It passes boundedness, topology, provenance, engagement, storage, and
head reporting. The proposed 0.9990 recall and 37.6 ms IVF values remain useful
comparison context but are not the basis of this decision. Please confirm that
the packet supports leaving production defaults unchanged and closing Task 180
as a measured negative result for this tuning direction.
