# Task 193: ec_distann Owner Payload Batch Fetch

Status: **complete — STOP** (2026-07-21).
Priority: P2. Roadmap candidates: `MAT-19`, `MAT-23`, `MAT-24` (pick one).

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`.

## Why

Task 187's 100k attribution measures `owner_payload_sql_work` at
8.340 ms/scan for a mean of 6.64 returned remote rows — ~1.26 ms per row.
That per-row cost points at per-candidate statement execution inside the
owner payload endpoint rather than one set-returning fetch per owner window.
After Task 192 removes (or fails to remove) the open/validate component,
this is the next-largest attributed owner-side cost.

## Goal

Advance exactly one bounded candidate that fetches a materialization
window's rows per owner in one batched operation instead of per-candidate
work. Eligible single candidates, chosen from measurement during
pre-registration:

- `MAT-19`: cache the owner-side inner SPI plan;
- `MAT-23`: direct batched `vec_id -> row-tier TID` lookup;
- `MAT-24`: `unnest(vec_ids) WITH ORDINALITY` join to directory/row tier
  with rank restoration.

Target the 8.34 ms/scan payload SQL component. Rank order, result identity,
tombstone/visibility handling, and the lazy10 window contract (FR-079,
ADR-085 D12, NFR-019 bounds including the `t`-skip qualifier) are unchanged.

## Reopened candidate

The packet-001 audit remains accepted: MAT-23/MAT-24 behavior is already
implemented, with one owner-window request and one ordinality-preserving
`unnest` query. The task remains open because MAT-19 is still in this task's
candidate pool. The explicitly pre-registered experiment is the narrow
owner-side prepared-plan cache for the `build_payload_sql` inner query,
keyed by generation plus projection fingerprint and invalidated with the
generation entry. It must be measured after the verified Task 194 A/A.

## Outcome

The isolated release 100k A/B preserved recall and storage but reduced warm
mean only `23.60 -> 23.50 ms` and owner payload SQL only
`8.746651 -> 8.599735 ms/scan`. MAT-19/MAT-20 therefore fail the
pre-registered usefulness gate and do not advance to full scale or
productionization. The packet-001 finding stands: MAT-23/MAT-24 batching is
already the production mechanism. Packet 005 contains the decision evidence;
Task 196 tracks the independent cache-off lazy10 duplicate found by the
optional promotion drill.

## Entry gate

Task 192 has a recorded disposition (PROMOTE or STOP). Sequencing is for
attribution only — this task does not depend on 192 winning; it depends on
192's counter movement being separately measurable from this one. Baseline
re-frozen on the then-current binary.

## Phases

1. Pre-register one candidate with predicted movement of
   `owner_payload_sql_work` and no movement elsewhere.
2. Isolated paired A/B at 100k on a byte-identical fresh generation, stage
   counters on; reject stage-local wins that do not move end-to-end
   mean/tails.
3. Standard 10k/50k/100k recall + latency + storage matrix via
   `ecaz bench suite` for an end-to-end useful candidate, plus the
   qual/projection/null/toast/tombstone and owner-outage drills (the batch
   path must abort identically on later-window failure).
4. PROMOTE to a separately reviewed productionization slice or STOP with the
   negative result recorded in the roadmap ledger.

## Required review packets

1. `reviews/task-193/001-preregistration-baseline/`;
2. `reviews/task-193/002-isolated-ab-100k/`;
3. `reviews/task-193/003-full-scale-decision/`.

## Non-goals

- Owner endpoint validation caching — Task 192.
- Wire/response format changes (packed buffers, binary protocol) — the
  batch fetch changes owner-internal execution only; `MAT-15`/`MAT-16`/
  `ARCH-07` remain separate.
- Traversal-path changes — Task 194.

## References

- `reviews/task-187/001-post-materialization-baseline/artifacts/` and
  Task 187 packet 004 round-2 feedback.
- Roadmap `MAT-19`, `MAT-23`, `MAT-24`; FR-079; FR-082; ADR-085 D12;
  NFR-019.
