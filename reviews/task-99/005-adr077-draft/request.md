# Review request — Task 99 packet 005: ADR-077 draft (Phase 4)

- Task: 99, AC 6 (ADR-077 PROPOSED → ACCEPTED at closeout)
- Coder: Task 102/103 author lane
- Date: 2026-06-11

## What this packet contains

`spec/adr/ADR-077-block-kernel-completeness-closing-record.md`
(PROPOSED) + index registration. The draft carries the three content
assignments from the pre-closeout architecture review
(`reviews/task-99/000-…/feedback/`):

- **F5** → §3 anchor/tolerance regime menu (four ratified regimes with
  preference order and when each applies);
- **F4** → §4 per-AM enablement policy (always-on / default-on /
  default-off, with the IVF default flip explicitly deferred to the
  profile data and taken at ACCEPT time);
- **F2** → §5 counter-key attribution (resolved by Task 101's kinds).

Plus: matrix-as-coverage-gate (§2), ISA dispatch policy incl. the new
`ecaz.isa_cap` and the G4 SVE2-vs-NEON measurement slot (§6), the two
structural lessons (§7), deliberate exclusions / honest bounds (§8),
and the two named open gaps (§9: SPIRE pq_fastscan product gap, HNSW
grouped-PQ coverage).

## Specific review asks

1. Are the four anchor regimes (§3) stated correctly per their source
   packets, and is the preference order right?
2. §4: is recording the IVF default decision as "taken at ACCEPT time,
   citing the profile" the right shape, or should the ADR not gate on
   it?
3. §6 leaves a bracketed slot for the G4 SVE2-vs-NEON outcome — flag if
   you'd rather the ADR not carry result-dependent text.
4. Anything from the F1–F9 findings that should be in the record and
   isn't.

## Status flow

PROPOSED now; the ACCEPT flip happens at Task 99 closeout (Phase 5)
together with the trip results (§6 slot) and the IVF default decision
(§4).
