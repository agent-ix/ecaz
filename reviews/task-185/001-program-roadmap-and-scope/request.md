---
task: 185
packet: 001-program-roadmap-and-scope
role: coder
status: open
date: 2026-07-19
head: aaa717879
---

# Review request: DistANN optimization roadmap and Task 185--190 split

Commit `aaa717879` creates the durable post-Task-183 optimization program
without turning the option inventory into one task or an unevidenced ADR.

## Review scope

1. `plan/design/ec-distann-recall-latency-roadmap.md` records the immutable
   Task 182/183 baseline, 150 stable candidate IDs across six workstreams, ten
   negative results, import rules, dependency gates, and ADR triggers.
2. Task 184 remains the executable measured latency priority and now links the
   `MAT-*` ledger.
3. Task 185 is an independently executable fixed-cap recall task. It changes
   the optimization objective from owner-seed representation to marginal
   success under the actual bounded traversal.
4. Tasks 186--190 are conditional:
   - 186 larger compressed/hierarchical heads after Task 185;
   - 187 traversal transport after Task 184 refreshes the residual profile;
   - 188 graph/search residual after entry work;
   - 189 codec work only after a same-seed trigger; and
   - 190 an architecture decision gate only after narrower work reports.
5. Every task selects at most one candidate and hands any production-affecting
   winner to a separately numbered implementation task.

The roadmap is intentionally not an ADR: it preserves alternatives and
negative evidence but selects no durable architecture. Each task identifies
the format/protocol/default decisions that would trigger an ADR later.

Please review candidate ownership, task boundaries, dependency gates, negative
result protection, and whether the split keeps each benchmark decision
attributable and reviewable.

## Validation

- Task numbers 185--190 were absent from the current checkout, `origin/main`,
  and remote matching task branches before creation.
- Exactly one canonical task file exists for every number 184--190.
- The ledger contains 40 `MAT`, 34 `HEAD`, 30 `TRAV`, 18 `GRAPH`, 13 `CODEC`,
  15 `ARCH`, and 10 `NEG` stable IDs.
- `git diff --cached --check` passed before commit.
- No test or benchmark was run because this checkpoint changes planning and
  task scope only; it changes no executable or production surface.

NFR-017's separate stakeholder-ruling reconciliation is already pushed as
Task 182 packet 008 and is a baseline prerequisite, not part of this candidate
selection request.
