# Task 216: ec_distann Owner Expansion and Serialization Latency

Status: **review pending — MAT-15 negative STOP; follow-up candidates blocked**
(2026-08-07).
Priority: P1 latency.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`.

## Why

Tasks 205 and 206 changed the interpretation of the latency residual.
Task 205 reduced traversal response bytes by roughly 52–65% with recall held
flat, but request bytes were unchanged and end-to-end latency moved only
modestly. Task 206's physical attribution likewise found only roughly 10–20 ms
of transport across about eight rounds against a much larger physical scan
time; the remaining gap is owner-side compute, response assembly, encoding,
and serialization rather than another generic network problem.

The old Task 201 residual task is superseded because its control used the
inadmissible coordinator full-graph replica. This task replaces only that
latency lane, with a conforming sharded owner control.

## Entry gate

1. Task 205's corrected bounded-L packet has a final disposition, so the
   response/threshold counters and candidate-limit semantics are fixed.
2. Task 206's accepted attribution is cited, including the distinction between
   transport wait and owner compute/serialization.
3. The control is a normal PG18 sharded owner-traversal release with the
   current production materialization/schema-cache behavior, no full-graph
   replica, and no attribution-only selector.
4. A fresh 100k physical generation and a checked-in `ecaz bench suite`
   configuration are available before implementation of a candidate.

This task may begin with diagnostic attribution after the entry evidence is
available. It must not stack its candidate with the Task 215 BW/H default
change in a decision cell. The MAT-15 isolated screen is STOPped: its
coordinator-side addressable ceiling is 0.19%, and its owner-SQL regression is
not a valid reason to reopen the family. MAT-21 remains blocked until the
generation identity and feature-build provenance issues are corrected.

## Goal

Identify and, at most, advance one owner-side expansion/serialization
optimization that improves end-to-end latency or tails without changing
results, placement, or failure semantics. The task is an attribution and
candidate-screening lane, not permission for a broad wire rewrite.

## Candidate screen

Pre-register at most three candidates from the measured dominant stage, then
advance at most one:

- `TRAV-05`: packed expansion responses instead of row/array structures;
- `TRAV-07`: contiguous packed neighbor codes and scores;
- `TRAV-23`: borrow graph/code bytes instead of allocating decode buffers;
- `MAT-15`: packed payload buffers with offsets and null bitmap; or
- `MAT-21`: typed/binary locators instead of textual locator formatting.

The selected candidate must be driven by stage evidence. Do not implement all
of these as a bundle, and do not assume that a response-byte reduction is an
end-to-end win after Task 205's result.

## Phases

1. **Attribution.** On a fresh 100k generation, reconcile owner graph read,
   neighbor scoring, response assembly, encoding, wire wait, coordinator
   decode, copy, and executor residual work. Report allocations/copies and
   request/response bytes separately. Reproduce the diagnostic on the
   candidate wide-beam point only as a labeled secondary view if Task 215 is
   active; it is not a mixed decision arm.
2. **Candidate pre-registration.** Name one candidate, predicted stage
   movement, invariants, expected work/byte movement, and the reason it could
   move end-to-end latency.
3. **Isolated A/B.** Measure the candidate against the unchanged conforming
   control at 100k first. Stop if the stage moves without end-to-end or tail
   movement, or if recall/result identity, memory, storage, or failure
   behavior changes.
4. **Full-scale decision.** Only a useful candidate proceeds to normal PG18
   10k/50k/100k recall, latency, storage, and topology evidence. A production
   winner receives a separate productionization task; a negative result closes
   the selected candidate family in the roadmap.

## Hard invariants

- Preserve FR-079 positional reassembly and deterministic tie ordering.
- Preserve Task 205 threshold and `L` semantics, including tie retention and
  tombstone/visibility handling.
- Preserve lazy payload windowing, projection/qual/null/toast behavior, and
  owner-outage failure semantics.
- Preserve NFR-021 distribution and NFR-022 control validity.
- Keep any packed or typed representation bounded, versioned where durable,
  and rejected safely when unsupported.
- Do not introduce a silent partial-result or owner-wide fallback.

## Benchmark gate

All matrices use checked-in `ecaz bench suite` configurations. The full-scale
gate reports recall/CI, ordered-result identity, mean/p50/p95/p99/max latency,
owner and coordinator stage work, allocations/copies where measurable,
request/response bytes, storage, build cost if affected, topology, failure
drills, and release provenance. Full-metrics diagnostic rows must be clearly
separate from normal benchmark decision rows.

## Non-goals

- Repeating generic transport or RTT work already closed by Tasks 187/194/206.
- Changing beam width, hop rounds, head construction, head selection, or
  degraded completion.
- Combining this candidate with Task 215's default-change A/B.
- Reopening Task 201's replica control or introducing coordinator full-graph
  state.
- A broad codec or persisted-format redesign without a separately accepted
  ADR and measured wire/decode justification.

## Required review packets

1. `reviews/task-216/001-attribution/`;
2. `reviews/task-216/002-isolated-candidate/`;
3. `reviews/task-216/003-full-scale-decision/`;
4. `reviews/task-216/004-closeout/`.

## References

- `plan/tasks/201-ec-distann-post-replica-latency-residual.md` (superseded
  control and reusable attribution decomposition);
- `plan/tasks/205-ec-distann-expansion-pushdown.md`;
- `plan/tasks/206-ec-distann-traversal-regime.md`;
- `reviews/task-205/004-l-bounded-rerun/`;
- `reviews/task-206/006-re-review-corrections/`;
- `reviews/task-206/007-scan-round-capture/`;
- roadmap candidates `TRAV-05`, `TRAV-07`, `TRAV-23`, `MAT-15`, and `MAT-21`;
- `spec/functional/distann/read/FR-079-distann-remote-expansion-protocol.md`;
- `spec/non-functional/NFR-019-distann-bounded-work.md`; and
- `spec/non-functional/NFR-021-distann-distribution-invariant.md`.
