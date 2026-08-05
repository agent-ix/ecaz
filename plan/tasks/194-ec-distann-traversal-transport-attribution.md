# Task 194: ec_distann Traversal Transport Attribution and One Candidate

> **MULTI-NODE MEASUREMENT RULE (NON-NEGOTIABLE).** Any decision about
> distributed behavior — latency, recall, storage, or overhead — MUST be measured
> on a multi-node configuration. A single-node / single-instance arm is NEVER
> acceptable as the basis for a decision about a distributed algorithm; its only
> permitted use is a clearly labeled baseline that quantifies distribution
> overhead. Label every reported number with its arm's node count. See
> AGENTS.md → "Distributed Measurement: Multi-Node Arms Only".

Status: **complete — STOP** (2026-07-22). Priority: P2. Inherits Task 187's
complete nine-way Phase 1 contract. Roadmap candidates: `TRAV-01` (complete),
then at most one of the families Task 187 listed.

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

## Outcome

The completion-audit release run delivered all 34 stage rows and 26 work rows
and passed both automatic reconciliation gates: remote expansion decomposed
within 1.17% and traversal total within 1.32%. At 100k, remote expansion was
7.429 ms/scan: owner service 2.259 ms, transport wait 5.013 ms, and only 0.071
ms combined connection/request-encode/client-decode work. Ten sequential
rounds, zero repeated nodes, and the dominant transport remainder confirm the
fixed-work wider/fewer-round family selected before the audit.

Packet 007's paired A/B remains the candidate decision: BW=8/H=50 improved
recall and reduced hops, traversal, and transport wait versus BW=4/H=100, but
warm mean moved only `24.30 -> 24.20 ms` and p95 regressed
`27.80 -> 28.30 ms` as expanded nodes and straggler spread rose.
TRAV-14/TRAV-15 therefore STOP without full-scale or productionization.
Packets 007 and 008 are the final decision and corrected-attribution evidence.

## Completion-audit reopening

Packet 007's candidate result remains valid, but the task disposition was
premature: the canonical run returned remote owner total service only, while
the `traversal_owner_graph_read` / `traversal_owner_score` rows still measured
coordinator-local expansion. Connection/prepared-state work, query-cache
hits, request/response bytes, remote response encoding, and client receive
decode were also absent, so the explicit nine-way Phase 1 contract was not
complete. Packet 008 added those feature-gated boundaries and an automatic
remote/traversal reconciliation gate. Its fresh canonical release run passed
both gates and accepted packet 007's candidate disposition unchanged.

## Constraints

- Preserve recall, ordering, epoch/failure semantics, and BW×H total work
  bounds; NFR-019 unchanged.
- Replicated layers/gateway copies (`TRAV-28`–`TRAV-30`) stay in Task 190;
  adaptive budgets (`TRAV-16`–`TRAV-19`) stay in Task 188.
- Protocol/format changes require a separate task and ADR review; a
  candidate requiring one outputs the ADR proposal, not the implementation.

## Phases and evidence

1. Counter slice: feature-gated traversal work counters and the nine non-overlapping
   attribution components above, including owner-side service and transport/straggler
   derivation, must be landed and reviewed before disposition.
2. Attribution run at 100k on a byte-identical fresh generation via a
   checked-in `ecaz bench suite` config with complete provenance and the 50/10 protocol.
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
