# Review request — Task 99 packet 008: Graviton 4 lane evidence

- Task: 99, item 9 G4 lane (+ Task 94 G4 pass + Task 97 G4 cells)
- Coder: Task 102/103 author lane
- Date: 2026-06-12

Complete G4 lane per the packet 006 runbook: day-one gate
(`sve2-128`, full parity set on real Neoverse V2), main profile
(91/91, 34/34 recall pairs byte-equal), the NEON-capped pass (cap
held, zero sve rows), and the Task 97 suite (14/14, `isa=sve2` rows).
Snapshot `snap-097eb8a8e881384dd`, stack destroyed. Source of truth:
`artifacts/manifest.md`.

## Headline for review

**SVE2 loses to NEON on G4 at every family where it dispatches**
(lut32 2.0–3.3×; e2e −27/−45% recoverable on TQ cells; control cells
identical). ADR-077 §6 records the dispatch-preference decision.
Please weigh in on:

1. The comparison methodology (capped vs default runs on identical
   fixtures/host; NEON-routed families as internal controls).
2. The Task 94 closure framing: its G4 evidence = the profile's
   grouped-pq cells (sve2 gather shape 144–160 ns/c, neon-capped
   119–130) — per its reopened-scope rule the gather-shape annotation
   applies, and the data additionally shows the SVE repack question is
   moot (NEON already wins). Task 94's status flip belongs to its
   owner citing these cells.
3. The two operational findings (pg_test skip on hosts; stale-snapshot
   catalog refresh via pg_get_functiondef replay) — both candidates
   for tooling hardening follow-ups.
