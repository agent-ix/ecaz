---
id: Plan-002
title: "ec_distann BatANN state-passing coordination (B0–B4)"
type: Plan
status: active
relationships:
  - target: ix://agent-ix/ecaz/FR-084
    type: references
  - target: ix://agent-ix/ecaz/FR-085
    type: references
  - target: ix://agent-ix/ecaz/FR-086
    type: references
  - target: ix://agent-ix/ecaz/FR-087
    type: references
  - target: ix://agent-ix/ecaz/FR-088
    type: references
  - target: ix://agent-ix/ecaz/FR-089
    type: references
  - target: ix://agent-ix/ecaz/NFR-021
    type: references
  - target: ix://agent-ix/ecaz/NFR-022
    type: references
---
# Plan-002: ec_distann BatANN state-passing coordination (B0–B4)

Executable plan for the BatANN coordination mode (ADR-086, reopening
ADR-085 D4): relay/baton query-state passing between data nodes as a
per-query GUC alongside the default coordinator loop, with stack and direct
return sub-modes, a depth budget with terminal hybrid resume, and a
three-way mode bench gate.

Normative sources: FR-084..FR-089 (`spec/functional/index/distann/`),
NFR-021..NFR-022, ADR-086 (D1–D11), TC-045..TC-048 (`spec/tests.md`), and
the **B0–B4 milestone table** in
`plan/design/batann-state-passing-coordination.md`. Operational task files:
`plan/tasks/173..178-*.md` (specs=173, B0=174 … B4=178). Spec review
reconciliation: `spec/reviews/batann/` (SR-008..SR-014).

## Requirements Summary

| Req | Title | Milestone(s) | Tests |
|---|---|---|---|
| FR-084 | Coordination Mode Selection (GUC, counters, degenerate equivalences) | B0 (surface), B1 (dispatch live) | TC-045, TC-046 |
| FR-085 | Relay-State Wire Format (v1, structural validation, budget authority) | B0 | TC-045 |
| FR-086 | Relay Endpoint + Local Drain (Algorithm 2) | B0 (local), B1 (remote) | TC-045, TC-046 |
| FR-087 | Stack-Mode Return (unwind, cancel, link failure, materialization fix) | B1, B3 drills | TC-046 |
| FR-088 | Direct-Mode Return (mailbox, at-most-once delivery, timeout=error) | B2, B3 drills | TC-047 |
| FR-089 | Depth Budget + Terminal Hybrid Resume | B1 | TC-046, TC-048 |
| NFR-021 | Relay Resource + Depth Bounds (occupancy, envelope, zero leaks) | B1–B3 | TC-046, TC-047, TC-048 |
| NFR-022 | Three-Way Mode Bench Gate (D9b bar, relay counters, D7 finding) | B4 | TC-048 |

## Dependency Graph

- `FR-085 state seam (B0) -> everything`
  Reason: the serializable `DistannBeamState` + local drain is the substrate
  for every relay behavior; the expansion-budget authority (D8) is fixed in
  its fields.
- `FR-084 GUC surface (B0) -> FR-086 dispatch (B1)`
  Reason: mode dispatch and counters need registered GUCs and the counter
  taxonomy before transport work.
- `FR-086 local drain (B0) -> FR-087 stack (B1) -> FR-088 direct (B2)`
  Reason: stack mode is transport wiring around the B0 endpoint; direct mode
  reuses the whole stack path and changes only the return.
- `Cancellation enabler (B1, shared-path) -> B1/B2 usability`
  Reason: nested relay awaits without interrupt slicing are uncancellable —
  a bench-host hazard; landed at B1 as its own slice (ADR-086 D10).
- `B1 kill-check verdict -> B2`
  Reason: under hash placement the structural stack-vs-coordinator
  comparison can already be negative at 2–3 nodes; the recorded
  proceed/de-scope verdict gates the mailbox investment (ADR-086
  Measurement Requirements).
- `Pre-B2 flush spike -> FR-088 shipped variant`
  Reason: send-and-abandon is likely unobtainable on stock tokio-postgres;
  the spike verdict selects send-and-abandon vs direct-lite before B2
  implementation.
- `B1+B2 machinery -> B3 fault matrix -> B4 gate`
  Reason: drills exercise both modes; the gate needs NFR-021 evidence and
  the drill-hardened fixture.
- `task-165 lane merge posture + relay-counter suite step (own commit) +
  task-172 protocol packet -> B4`
  Reason: NFR-022 prerequisites; the `distann-pipeline` step kind cited by
  NFR-017/TC-044 does not exist yet.

## Critical Path

B0 (Task-001) → B1 (Task-002) → B2 (Task-003) → B3 (Task-004) → B4 gate
(Task-005). Single-coder serial by design. Parallelizable: the suite-runner
relay-counter extension (Task-006, Track B) must merge before B4.

## Shared Dependencies (discrete deliverables)

- **`DistannBeamState` + `distann_local_drain`** — built at B0; single
  writer: B0 owns the field set (incl. expansion-budget authority) and the
  append-only-beam invariant guard test.
- **Relay counter taxonomy** (FR-084 normative list incl.
  `relay_depth_histogram`, `state_bytes_max/total`, `relay_journeys`) —
  registered at B0, emitted from B1, consumed verbatim by the B4
  results.jsonl schema.
- **Cancellation enabler** (interrupt-sliced awaits + CancelToken, SPIRE
  dispatch port) — B1 own slice; benefits coordinator mode.
- **Suite-runner relay extension** (coordination-mode axis + counter
  emission) — Task-006, lands as its own commit before B4 (FR-038 rule).

## Quality Gates

#### Gate G0: B1 kill-check (ADR-086 Measurement Requirements) — gates B2
- **Measures:** informational stack-vs-coordinator latency + relay-rate at
  2/3 nodes on the fixture, release build.
- **Pass criteria:** recorded proceed verdict (stack mode not structurally
  dominated at gate-relevant BW/H, or a stated reason to continue).
- **If fails:** de-scope = defer direct mode (B2/B3 mailbox work), keep
  stack mode for the B4 record.

#### Gate G1: pre-B2 flush spike (ADR-086 D4)
- **Measures:** whether send-and-abandon can guarantee statement flush on
  the pooled tokio-postgres transport.
- **Pass criteria:** a recorded verdict either way; direct-lite is the
  planned fallback (FR-088 wording is variant-scoped).
- **If fails:** ship direct-lite; NFR-022 gate packet records the variant.

#### Gate G2: B4 bench gate (program gate)
- **Measures:** pre-registered three-way mode matrix at 10k/50k/100k
  (NFR-022): D9b one-sided recall bar, per-mode p50/p95, relay counters,
  pinned reduced-depth row, D7 relay-rate finding.
- **Pass criteria:** NFR-022 thresholds; promote/iterate/shelve verdict
  written into ADR-086 status.
- **Prerequisite:** task-165 merge posture recorded; Task-006 landed;
  task-172 protocol packet.

## Test Plan

| Test | What | Harness | Milestone |
|---|---|---|---|
| TC-045 | State round-trip + append-only-beam guard + structural-bounds/version/fingerprint rejection + no-heap_tid/no-conninfo inspection + GUC drills + single-node relay identity + amgettuple dispatch | unit/pg_test (`relay_state.rs`, `src/tests/ec_distann_relay.rs`) | B0 |
| TC-046 | Stack-mode fixture drills: D9a identity, drain-all-local-first, handoff target, occupancy at held depth, cancel + link-failure teardown, depth-exhaustion terminal resume, depth-0 equivalence, delta-buffer seam, full-mesh check, fault classification | loopback + real multinode fixture | B1/B3 |
| TC-047 | Direct-mode drills: latch wakeup, stack≡direct, error-delivery per class, timeout (no rerun), slot lifecycle + exhaustion fallback, oversize, zero leaks | multinode fixture | B2/B3 |
| TC-048 | Three-way gate matrix + relay counters + D7 row + reduced-depth row | `ecaz bench suite` | B4 |

## Remaining Work

### Track A: Critical Path (serial)
Task-001 (B0) → Task-002 (B1, G0) → Task-003 (B2, G1) → Task-004 (B3) →
Task-005 (B4, G2).

### Track B: Parallel
Task-006 (suite-runner relay extension) — merge deadline: before Task-005.

## Task File Mapping

| Task file | Track | Milestone / repo task | Owns | Status |
|---|---|---|---|---|
| Task-001-b0-state-seam.md | A | B0 / `plan/tasks/174` | FR-085, FR-084(surface), FR-086(local) | not_started |
| Task-002-b1-stack-mode.md | A | B1 / `plan/tasks/175` | FR-086(remote), FR-087, FR-089, cancellation enabler, G0 | not_started |
| Task-003-b2-direct-mode.md | A | B2 / `plan/tasks/176` | FR-088, G1 | not_started |
| Task-004-b3-faults.md | A | B3 / `plan/tasks/177` | NFR-021 evidence, fault matrix | not_started |
| Task-005-b4-bench-gate.md | A | B4 / `plan/tasks/178` | NFR-022, G2 | not_started |
| Task-006-suite-runner-relay-extension.md | B | pre-B4 / `plan/tasks/178` prereq | mode axis + relay-counter emission | not_started |

## Coordination Rules

- One coder, one branch per milestone task (`task-17N-…`); packets under
  `reviews/task-17N/`; branch lineage stays on the distann lane until
  task-165 merges (record the posture in each packet).
- **Freeze the `DistannBeamState` field set and counter taxonomy at B0**;
  B1+ implement against them without renegotiation unless G0/G1 fail.
- Do not start Task-003 before G0's recorded verdict; do not start B2
  implementation before G1's spike verdict.
- No implementation before the Task 173 spec packet is accepted.
- Bench discipline per CLAUDE.md: `ecaz bench suite` only, A/B per change,
  10k/50k/100k, release-verified backend, evidence packet-local.
