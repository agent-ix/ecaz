---
task: 182
packet: 001-entry-and-production-design
role: coder
status: review-requested
head: d6a1ca4507f441467c393183dd7a32eb3e776142
date: 2026-07-15
---

# Review request: conditional bounded-head production task

Please review the proposed Task 182 definition at
`plan/tasks/182-ec-distann-bounded-head-production.md` and its task-index entry.
This checkpoint reserves and constrains a later production task; Task 182 is
explicitly non-startable unless Task 181 first records an outside-reviewed
full-scale GO for one frozen bounded candidate.

## Boundary being reviewed

Task 180 requires proven benchmark strategies to be implemented and remeasured
under a separately numbered task. Task 181 owns coverage/landmark/hierarchy
diagnosis and candidate selection without production changes. Task 182 owns
only the translation of Task 181's final winner into production format,
generation lifecycle, and query paths.

If Task 181 closes NO-GO, Task 182 becomes `won't pursue`; it must not invent a
new candidate or use a promising intermediate result as its entry gate.

## Requested review decisions

1. Does the seven-item entry gate prevent implementation before Task 181 has a
   fully specified, full-scale, outside-reviewed winner?
2. Are deterministic construction, format/fingerprint compatibility, bounded
   builder memory, query caps, and generation lifecycle addressed before scan
   wiring?
3. Do the fail-closed requirements prevent hidden owner scans, uncapped remote
   seeding, and silent old-index reinterpretation?
4. Is the PG18 correctness/recovery matrix sufficient for new head artifacts
   across Ready/publish/retire/reclaim and concurrent scan pins?
5. Does the production-path 10k/50k/100k A/B require independent reproduction
   rather than accepting Task 181's benchmark-feature result?
6. Are promote/iterate/abandon and default-change rules conservative when a
   correct implementation fails to reproduce the measured Pareto point?

## Validation

- `git diff --check 5fbb37f46..d6a1ca450`: pass.
- The Task 181 definition and all referenced Task 179/180 and FR/NFR sources
  exist in this checkout.
- No tests or benchmarks were run because this checkpoint changes planning
  Markdown only.

Please leave the outside decision under this packet's `feedback/` directory.
