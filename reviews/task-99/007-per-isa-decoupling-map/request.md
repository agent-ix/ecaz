# Review request — Task 99 packet 007: per-ISA comparison + decoupling map (Phase 3, interim)

- Task: 99, AC 3/4 + scope item 5
- Coder: Task 102/103 author lane
- Date: 2026-06-12

`artifacts/per-isa-comparison-and-decoupling-map.md`: the Phase 3
deliverable in interim form — local-Intel (packet 003) and Apple-M5
(task-104/008) columns final; G4 SVE2 + G4 NEON-capped + AWS-Intel
columns land with the trip and complete it.

Highlights for review:

1. **Decoupling classes A–D** from the first single-host all-cells
   dataset: class A scoring-dominated (IVF TQ −66/−69% headline),
   class B pruning-trade — including the **first measured negative-net
   batch cell** (IVF QJL @1024, +8.3%/+3.0% at nprobe 8/16), which is
   direct input to the IVF GUC default decision (ADR-077 §4); class C
   other-stage-dominated (rerank/traversal/pipeline); class D
   no-axis cells.
2. The pre-trip per-ISA observation that AVX2 and M5-NEON land within
   ~±15% on most families despite the 2× vector-width difference —
   sanity-check the framing.
3. The cross-host SPIRE note (class A locally at 100k, mild on M5 at
   10k — fixture-scale, not ISA).

Ask: confirm the class assignments and the negative-net IVF QJL cell
reading (small-fixture/small-nprobe caveat noted) before the trip data
locks the table.
