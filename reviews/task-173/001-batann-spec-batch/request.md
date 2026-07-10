# Review Request: Task 173 — BatANN Spec Batch (State-Passing Coordination)

- Task: `plan/tasks/173-batann-spec-authoring.md`
- Branch: `task-173-batann-specs` (based on `task-165-ec-distann-m3`)
- Date: 2026-07-09
- Role: coder (spec authoring; no implementation in this batch)

## What this packet covers

The complete planning batch for the BatANN coordination mode of
`ec_distann` — relay/baton query-state passing between data nodes (paper
arXiv:2512.09331), reopening ADR-085 D4:

1. **Design doc** — `plan/design/batann-state-passing-coordination.md`
   (architecture, protocol mapping onto the PostgreSQL execution model,
   normative B0–B4 milestone table, reuse map, hazard analysis).
2. **ADR-086** — `spec/adr/ADR-086-ec-distann-batann-state-passing.md`
   (D1–D11: GUC mode surface; relay-state wire format v1; stack = nested
   SQL unwind; direct = shmem mailbox with at-most-once delivery and
   send-and-abandon spike / direct-lite fallback; transport-pool
   generalization; min(H,16) depth default + terminal hybrid resume;
   hash-placement relay-rate caveat; expansion budget travels in the state;
   two-bar result equivalence; cancellation authority; endpoint auth
   posture).
3. **FR-084..FR-089** — `spec/functional/index/distann/`
   (mode selection; relay-state wire format; relay endpoint + Algorithm 2
   local drain; stack return; direct return; depth budget + hybrid resume).
4. **NFR-021..NFR-022** — relay resource/depth bounds; three-way
   coordination-mode bench gate.
5. **Test matrix** — TC-045..TC-048 + coverage and permutation rows in
   `spec/tests.md`; indexes updated (`spec/functional/index.md`,
   `spec/non-functional/index.md`, `spec/adr/index.md`).
6. **Spec review (seven dimensions)** — `spec/reviews/batann/`
   (SR-008 failure-domain, SR-009 integrity, SR-010 dependency, SR-011
   evidence, SR-012 risk-complexity, SR-013 scope-boundary, SR-014 base):
   69 findings (13 high), **all dispositioned** in per-file Reconciliation
   sections; the spec fixes are in this same batch.
7. **Task program** — `plan/tasks/173-178` (+ README index rows) and the
   TDD plan bundle `plan/Plan-002-batann-state-passing/` (6 tasks, gates
   G0 = B1 kill-check gating B2, G1 = pre-B2 flush spike, G2 = B4 gate).

## Key review-driven decisions to check

- At-most-once mailbox delivery: never-reused 64-bit query_ids, delivery
  rights travel with the state, timeout = classified error (no rerun),
  slot-exhaustion → transparent coordinator-mode fallback (FR-088).
- FR-085 structural validation of received states (distrust posture) and
  the expansion budget as the authoritative bound (rounds derived).
- `relay_max_depth` default min(H, 16) — H=100 default made depth=H unsafe
  (101 backends worst case); NFR-021 carries the arithmetic (≈166 KB state
  envelope at shipped defaults).
- Terminal hybrid resume (no re-relay after depth exhaustion).
- ADR-086 D11 endpoint auth (EXECUTE revoked from PUBLIC).
- Shared-path changes named: cancellation enabler at B1 (fixes
  coordinator-mode uncancellability), batann-scoped materialization
  relaxation.
- B4 merge prerequisites recorded (task-165 lane posture, relay-counter
  suite step as own commit, task-172 protocol packet).

## Validation

- `quire validate --scope . "spec/**/*.md"` and
  `"plan/Plan-002-batann-state-passing/**/*.md"` clean (only pre-existing
  advisory EARS warnings on earlier distann files).
- No `src/**` or `crates/**` changes in this batch (planning only;
  implementation starts at Task 174).

## Requested review

Spec-level review of the batch, with particular attention to the "Key
review-driven decisions" above and anything the seven-dimension pass
missed. Open the packet's feedback/ with findings; do not close.
