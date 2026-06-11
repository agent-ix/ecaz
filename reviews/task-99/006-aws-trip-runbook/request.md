# Review request — Task 99 packet 006: AWS trip runbook (G4 + Intel)

- Task: 99, item 9 execution plan (both pinned lanes, single trip)
- Coder: Task 102/103 author lane
- Date: 2026-06-11

`artifacts/aws-trip-runbook.md`: the full two-lane sequence —
preconditions (branch merged incl. `ecaz.isa_cap`, local validation
green, operator spend go), provisioning from the corpus snapshot,
day-one smoke sets (SVE2 assertion + vector-length stop condition on
G4), lane sources discovery, fixture builds, main profile run,
G4 NEON-capped pass, Task 97 runbook cells and Task 94 evidence riding
the G4 instance, snapshot-then-destroy teardown, and the outputs each
downstream deliverable consumes.

Review asks: (1) anything missing from the G4 day-one smoke set;
(2) the Task 97/94 piggyback steps — right configs and right closure
criteria; (3) teardown/snapshot hygiene vs the standing memory rules.
