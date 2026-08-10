# Task 220: ec_distann Owner Array Materialization (MAT-16)

Status: **complete — review-closed ACCEPT, STOP** (2026-08-10; round-2
feedback `reviews/task-220/002-isolated-candidate/feedback/2026-08-10-01-reviewer.md`).
The pre-registered MAT-16 screen's STOP is accepted (payload SQL 9.36→32.06
ms/scan, ~3.4×; predictions byte-identical). The P0 correction `c8b5fd9ee` is
verified: featureless production and FR-079 use `build_payload_sql`, the
packed representation is benchmark-only, and the production SQL shape is
test-pinned. MAT-16 is rejected as implemented (chained-concat/`octet_length`
SQL form); any revised packed representation needs a new numbered task.
Priority: P1 latency.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`.
Origin: Task 218 carry-in; new task, not a reopening of Task 218.

## Goal

Measure and, if useful, implement MAT-16: avoid PostgreSQL array construction
for each owner payload row in the production lazy-10 materialization path.
The task must establish whether this changes the remaining owner-side payload
SQL stage without changing recall, ordering, storage conformance, or the
shipped defaults.

## Why

Task 218 measured the production lazy-10 owner endpoint at 9.10 ms/scan and
owner payload SQL at 8.555 ms/scan for the MAT-21 control. Its typed-locator
candidate was neutral and retired, but the owner payload SQL stage remains
unresolved. MAT-16 is therefore a distinct owner-side hypothesis and needs
its own attribution; MAT-21's negative is not evidence for or against it.

## Entry gate

1. Task 217's same-generation attestation lane remains the required identity
   gate for every physical arm.
2. Task 218's production lazy-10 denominator and MAT-21 STOP are accepted;
   this task keeps the shipped implementation and defaults as its control.
3. The candidate arm and all measurable predicates are pre-registered before
   any result is inspected.

## Scope

### P1 — Isolated candidate screen

Run a production lazy-10 control/candidate A/B at 100k with one binary/runtime
generation identity across the arms. Change only the owner-side array
construction mechanism. Capture recall, prediction bytes, warm latency,
`custom_scan_total`, owner payload SQL/endpoint counters, storage, and
NFR-021/NFR-022 conformance.

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
- Implementing MAT-22's expanded-candidate locator path.
- Changing beam width, head construction, search budget, or shipped defaults.
- Using eager `materialization_batch_size=0` as the production denominator.

## Acceptance

1. The candidate is isolated from the control with one generation identity and
   packet-local structured `results.jsonl` evidence.
2. A/A prediction identity and recall-safety checks pass; any A/B movement is
   attributable to MAT-16 rather than a generation or query-surface change.
3. The task records either a justified STOP or the complete 10k/50k/100k
   matrix with recall, latency, storage, and NFR conformance.
4. The task header, README row, and MAT-16 roadmap entry record the reviewed
   disposition.

## Required review packets

1. `reviews/task-220/001-preregistration-and-screen/`
2. `reviews/task-220/002-isolated-candidate/`
3. `reviews/task-220/003-release-matrix-and-decision/` (only if 002 is useful)

## References

- `reviews/task-218/001-production-profile-attribution/`
- `reviews/task-218/002-isolated-candidate/`
- `reviews/task-217/002-lane-implementation/`
- Roadmap MAT-16 / MAT-21 / MAT-22 candidate ledger
