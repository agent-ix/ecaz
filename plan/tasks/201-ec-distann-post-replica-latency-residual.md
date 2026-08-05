# Task 201: ec_distann Post-Replica Latency Residual

> **MULTI-NODE MEASUREMENT RULE (NON-NEGOTIABLE).** Any decision about
> distributed behavior — latency, recall, storage, or overhead — MUST be measured
> on a multi-node configuration. A single-node / single-instance arm is NEVER
> acceptable as the basis for a decision about a distributed algorithm; its only
> permitted use is a clearly labeled baseline that quantifies distribution
> overhead. Label every reported number with its arm's node count. See
> AGENTS.md → "Distributed Measurement: Multi-Node Arms Only".

Status: **SUPERSEDED by Tasks 205 and 206** (2026-07-29). Do not start this task
as written.

> **Why superseded.** This task's Frozen control section (below) places the Task
> 199 coordinator traversal replica *inside* the control, and its scope rules
> forbid replica format/design questions from entering the screen. Task 203's
> audit found the replica holds every owner's graph record and full-precision
> vector on one coordinator (1.660 GB at 100k, linear in N), so it does not
> satisfy `NFR-021` and is inadmissible as a decision control under `NFR-022`. A
> latency attribution run against it would measure a single-node index and
> produce another uninterpretable result — and as written this task could not
> surface that, because the questions are ruled out of scope.
>
> The latency lane is now **Task 205** (Algorithm 1 expansion pushdown) followed
> by **Task 206** (traversal regime), both controlled against the
> owner-traversal arm. Whatever residual remains after 206 reports is attributed
> there, or in a successor task pinned to a conforming control.
>
> Evidence: `reviews/task-203/001-decision-reaudit/` and
> `reviews/task-201/001-control-validity-supersession/`.
>
> The material below is retained unchanged for history. Its Phase 1 attribution
> decomposition is still a good contract and should be reused by 205/206; only
> its control is invalid.

Priority: P1 latency follow-up after Task 199 (historical).

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`. This task
owns the post-replica latency attribution and one isolated optimization; it
does not reopen the recall/head program owned by Tasks 185--186.

## Why

Task 199 already supplied the decision-grade aggregate latency baseline. The
normal coordinator traversal replica reduced warm mean from 18.3/20.4/19.9 ms
to 15.3/16.4/16.2 ms at 10k/50k/100k, with exact recall and unchanged storage
between the owner and replica arms. Task 200 found and fixed a benchmark-only
allocation-retention defect without changing the production read path.

Those results do not identify the remaining latency stage after local replica
traversal. Repeating the Task 199 A/B unchanged would add no information. The
next question is which remaining bounded stage—remote payload materialization,
executor/copy work, or local traversal—can produce a useful end-to-end win.

## Goal

Using the accepted Task 199 normal-replica release as the control, attribute the
remaining latency and advance at most one isolated latency candidate. A useful
candidate receives a separate 10k/50k/100k release A/B and productionization
task. A negative result closes the selected family without changing defaults.

## Frozen control

The control is the current accepted production point:

- Task 199 normal-build coordinator traversal replica, when an explicit Ready
  image exists, with unchanged owner fallback;
- trained 4,096-landmark head;
- BW4/H100 traversal budget;
- RaBitQ neighbor values and exact final ranking;
- Task 195 owner schema-cache behavior; and
- Task 196 identity-keyed lazy10 payload materialization.

No Task 185/186 head change, Task 188 BW8 change, graph rebuild, neighbor
codec change, replica format/lifecycle change, or new benchmark selector may
enter the control or candidate screen.

Task 199's accepted release rows are the aggregate baseline. The first
attribution screen must use a fresh committed PG18 release on a byte-identical
100k physical generation, with the normal replica path and the owner fallback
control both labeled. Diagnostic counters may be enabled only in a separately
labeled instrumentation run; they are not product latency evidence.

## Phase 1: residual attribution

Measure, reconcile, and report at least:

1. local replica graph traversal and frontier/merge work;
2. owner-side payload SQL, heap lookup, detoast, encode, and transport wait;
3. coordinator decode, datum ownership, copy, output association, and executor
   residuals;
4. connection/session setup and reuse;
5. remote payload rows/bytes, exact reads, candidate work, and tails; and
6. RSS and allocation behavior under the fixed normal path, reusing Task 200's
   bounded-memory gate where applicable.

The screen must distinguish owner fallback from replica selection and must not
add failed replica work to the successful owner/replica latency total. Use the
Task 199 release numbers as the historical control; do not claim a new gain
until a candidate A/B is run on the same generation and query identities.

## Candidate screen

Pre-register at most three diagnostic candidates from the measured dominant
stage, then advance at most one:

- payload/executor candidates: MAT-07, MAT-08, MAT-11, MAT-12, MAT-14,
  MAT-15, MAT-21, MAT-25, MAT-26, MAT-39, or MAT-40;
- local traversal candidates: TRAV-02, TRAV-03, TRAV-04, TRAV-05, TRAV-08,
  TRAV-11, TRAV-12, TRAV-20, TRAV-21, TRAV-22, TRAV-23, TRAV-24, or TRAV-25;
- tail/accounting candidate: TRAV-27, only if straggler or owner scheduling
  attribution is material.

Do not stack payload, traversal, and architecture changes. Do not import
ARCH-11/13/14/15, a new coordinator replica design, or BW8 into this task.
Those require separate workload/topology or current-production-head premises.

## Evidence and decision

All matrices and screens use checked-in `ecaz bench suite` configurations.
The 100k attribution screen may stop without a full matrix if no candidate
passes the usefulness gate. Only one useful candidate proceeds to a release
A/B at 10k/50k/100k with recall, mean/p50/p95/p99/max latency, storage,
construction or rebuild cost, remote engagement, query separation, topology,
and clean release provenance. The release A/B must use the normal production
path and preserve Task 199's exact result identity and fallback semantics.

Advance only if the candidate improves end-to-end latency or tails without a
material recall, storage, build, memory, topology, or failure-semantics cost.
Any production-affecting winner is handed to a separately numbered
productionization task and ADR/spec work when its lifecycle or protocol
contract changes.

## Required review packets

1. `reviews/task-201/001-post-replica-attribution/`;
2. `reviews/task-201/002-isolated-latency-candidate/`;
3. `reviews/task-201/003-release-matrix-and-decision/`;
4. `reviews/task-201/004-closeout/`.

## Non-goals

- Recall/head selection, landmark training, or 4,096→8,192 capacity work;
- BW8 or any other search-budget/default change;
- graph construction, neighbor codec, or persisted replica-format changes;
- repeating Task 199's already accepted aggregate replica A/B; and
- cross-ISA portability, which belongs to Task 202.

## References

- Task 199 release matrix and outside review;
- Task 200 memory-retention regression and Task 188 follow-up;
- Tasks 191, 195, and 196 production payload/materialization work;
- `plan/design/ec-distann-recall-latency-roadmap.md`; and
- NFR-007 and NFR-017 through NFR-020.
