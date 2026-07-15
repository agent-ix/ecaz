---
task: 180
packet: 003-full-scale-decision
role: coder
status: open
date: 2026-07-15
---

# Review request: full-scale bounded-head confirmation

Please review the registered Phase 2 matrix and, once measurements land, the
Task 180 GO/NO-GO decision.

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
4. Check the final NFR-017 verdict against recall >=0.9990 at every scale,
   100k p50 <=37.6 ms, and p95 <=3x its own p50.

## Next action while review is open

Audit/dry-run the checked-in suite, then execute each scale separately with
disk-safe run-directory cleanup and append decision-grade artifacts/results.
