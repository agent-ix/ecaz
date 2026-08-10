# Task 221: ec_distann Owner Expanded Locator (MAT-22)

Status: **implementation complete; packet 002 review-open; measured STOP pending reviewer disposition** (2026-08-10). Priority: P1 latency.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`.
Origin: Task 218 carry-in; new task, not a reopening of Task 218.

## Goal

Measure and, if useful, implement MAT-22: return the row-tier locator with
expanded candidates so the owner materialization path can remove a lookup.
The task must establish whether this changes the owner expansion/wire path
without changing recall, ordering, storage conformance, or shipped defaults.

## Why

Task 218 measured the production lazy-10 owner endpoint at 9.10 ms/scan and
retired MAT-21 after the typed-locator A/B was neutral. MAT-22 targets a
different boundary—owner expansion and locator transport—and was not tested
by that task. It remains an open candidate with no measured win to assume.

## Entry gate

1. Task 217's same-generation attestation lane remains the required identity
   gate for every physical arm.
2. Task 218's production lazy-10 denominator and MAT-21 STOP are accepted;
   this task keeps the shipped implementation and defaults as its control.
3. The expanded-candidate payload shape, ordering contract, and failure
   behavior are pre-registered before any result is inspected.

## Scope

### P1 — Isolated candidate screen

Run a production lazy-10 control/candidate A/B at 100k with one binary/runtime
generation identity across the arms. Change only the owner expansion/wire
path that returns the row-tier locator with expanded candidates. Capture
recall, prediction bytes, ordering, warm latency, `custom_scan_total`, owner
endpoint/lookup counters, storage, and NFR-021/NFR-022 conformance.

### P2 — Decision

If the candidate is neutral or regresses on the pre-registered end-to-end
contract, close with STOP and no release matrix. If it is useful end to end,
run the standard 10k/50k/100k recall + latency + storage A/B matrix through
`ecaz bench suite`, with the same-generation and production lazy-10 controls.

### P3 — Follow-up boundary

A measured win is not a default or release change. Any productionization,
rollback, or shipped-default decision requires a separately numbered task.

## Non-goals

- Reopening or re-measuring MAT-21's typed/binary locator hypothesis.
- Implementing MAT-16's owner array-construction path.
- Changing beam width, head construction, search budget, or shipped defaults.
- Using eager `materialization_batch_size=0` as the production denominator.

## Acceptance

1. The candidate is isolated from the control with one generation identity and
   packet-local structured `results.jsonl` evidence.
2. A/A prediction identity, recall safety, and ordering checks pass; any A/B
   movement is attributable to MAT-22 rather than a generation or query-
   surface change.
3. The task records either a justified STOP or the complete 10k/50k/100k
   matrix with recall, latency, storage, and NFR conformance.
4. The task header, README row, and MAT-22 roadmap entry record the reviewed
   disposition.

## Required review packets

1. `reviews/task-221/001-preregistration-and-screen/`
2. `reviews/task-221/002-isolated-candidate/`
3. `reviews/task-221/003-release-matrix-and-decision/` (only if 002 is useful)

## References

- `reviews/task-218/001-production-profile-attribution/`
- `reviews/task-218/002-isolated-candidate/`
- `reviews/task-217/002-lane-implementation/`
- Roadmap MAT-16 / MAT-21 / MAT-22 candidate ledger
