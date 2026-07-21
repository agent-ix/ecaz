# Task 194: ec_distann Traversal Transport Attribution and One Candidate

Status: **in progress** (2026-07-21). Priority: P2. Inherits Task 187's
complete nine-way Phase 1 contract. Roadmap candidates: `TRAV-01` (active
prerequisite), then at most one of the families Task 187 listed.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`.

## Why

Task 187 closed STOP: traversal is 7.468 ms of the 22.40 ms 100k warm mean,
remote owner expansion is 6.174 ms of that (493 samples ≈ 9.9 remote
expansions/query at 0.626 ms each vs 0.155 ms per local expansion), so
roughly 4.6 ms/query is transport/encode/wait overhead by the local-proxy
derivation — but no counter separates encode, owner work, wire wait,
decode, or straggler spread on the traversal path. The materialization path
already has that decomposition (Tasks 192/193 act on it); the traversal path
does not.

## Goal

Phase 1: implement the feature-gated nine-way traversal decomposition
exactly as contracted in Task 187:

1. coordinator frontier and owner partitioning;
2. connection/session/prepared-state work;
3. request encoding and bytes;
4. owner directory/graph reads and decode;
5. owner approximate/exact scoring;
6. response encoding and bytes;
7. transport wait and per-owner straggler distribution;
8. coordinator receive/decode/frontier insertion; and
9. hop count, expansion batch widths, nodes requested/returned, cache hits,
   and repeated node/page reads.

Counters are feature-gated (NFR-020 taxonomy, off by default), reset after
warmups, non-overlapping, and must reconcile against `remote_expand` /
`local_expand` / `traversal_total` within a stated tolerance.

Phase 2: pre-register at most one candidate from the measured decomposition.
Given ~9.9 sequential expansion rounds per query, hop fusion/pipelining
(`TRAV-08`–`TRAV-15`) is the likely family if round-trip wait dominates;
caching (`TRAV-02`–`TRAV-04`) or packed responses (`TRAV-05`–`TRAV-07`) if
decode/repeat-read dominates. Do not choose before the counters say.

## Entry gate

- Counter implementation (Phase 1) may start immediately: it is
  feature-gated instrumentation with no production behavior change, reviewed
  as its own slice.
- The attribution run and any candidate A/B wait until Tasks 192/193 have
  recorded dispositions, so the traversal baseline is the then-current
  production binary and no two changes share an A/B window.

## Constraints

- Preserve recall, ordering, epoch/failure semantics, and BW×H total work
  bounds; NFR-019 unchanged.
- Replicated layers/gateway copies (`TRAV-28`–`TRAV-30`) stay in Task 190;
  adaptive budgets (`TRAV-16`–`TRAV-19`) stay in Task 188.
- Protocol/format changes require a separate task and ADR review; a
  candidate requiring one outputs the ADR proposal, not the implementation.

## Phases and evidence

1. Counter slice: traversal hop/batch/request/response/frontier/repeat work
   counters landed behind the measurement feature; owner timer decomposition
   and reconciliation remain before candidate selection.
2. Attribution run at 100k on a byte-identical fresh generation via a
   checked-in `ecaz bench suite` config; nine non-overlapping components
   published.
3. At most one pre-registered candidate, isolated paired A/B at 100k; then
   the standard 10k/50k/100k recall + latency + storage matrix if
   end-to-end useful.
4. PROMOTE to separate productionization or STOP with the ledger updated.

## Required review packets

1. `reviews/task-194/001-traversal-counters/`;
2. `reviews/task-194/002-nine-way-attribution/`;
3. `reviews/task-194/003-isolated-candidate/`;
4. `reviews/task-194/004-full-scale-decision/`.

## Non-goals

- Materialization-path work — Tasks 192/193.
- Graph construction/adaptive search — Task 188. Codec — Task 189.
  Architecture — Task 190.

## References

- Task 187 packets 001–004 and feedback (STOP rationale, nine-way contract,
  transport-share derivation).
- Roadmap `TRAV-01`–`TRAV-27`; FR-082; ADR-085; NFR-019/NFR-020.
