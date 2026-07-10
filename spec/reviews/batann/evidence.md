---
id: SR-011
title: Evidence Analysis of the ec_distann BatANN Spec Batch
type: SpecReview
analysis: evidence
scope: "spec/functional/index/distann/FR-084..089, spec/non-functional/NFR-021..022, spec/adr/ADR-086, spec/tests.md TC-045..048, plan/design/batann-state-passing-coordination.md"
review_set: all
---

## Summary

Verification-and-evidence pass over every Acceptance Criteria row of
FR-084..FR-089, the Measurement and Evaluation tables and Verification
sections of NFR-021..NFR-022, the TC-045..TC-048 matrix / coverage /
option-permutation rows in `spec/tests.md`, ADR-086's Measurement
Requirements, and the B0–B4 milestone table in
`plan/design/batann-state-passing-coordination.md`. Precedent applied:
SR-005 (`spec/reviews/evidence.md`) and the FR-079/FR-081/NFR-019 →
TC-040/041/044 mappings; repo evidence rules (`ecaz bench suite` only,
NFR-007 provenance, packet manifests).

What holds up: every AC of the six FRs carries an explicit verification
method and traces to a TC row (FR-084→TC-045/046, FR-085→TC-045,
FR-086→TC-045/046, FR-087→TC-046, FR-088→TC-047, FR-089→TC-046/048); the
NFR-021/NFR-022 measurement rows each name a concrete channel (fixture
drills, counter assertions, suite steps) rather than "TBD"; the D8
budget-travels-in-state design makes the global BW×H counter assertions
(FR-086-AC-4, FR-089-AC-4, NFR-021 row 5) genuinely producible, because the
counters ride the state and the terminal state carries the cross-node
totals; the design doc's B4 row explicitly promises the suite-runner
coordination-mode axis and relay-counter emission into results.jsonl, so the
NFR-022 evidence chain (suite config in packet → suite-manifest.json →
results.jsonl fields) is complete on paper; `distinct_recall` is present in
this worktree's `crates/ecaz-cli/src/commands/bench/suite.rs`, so SR-005
FND-001's branch-residency blocker does not recur for TC-048; and NFR-022's
scope pins the multinode cells to the real multi-instance
distann-local-multinode / task-172 protocol, correctly reserving the
loopback fixture for the TC-046/047 correctness drills.

The findings concentrate on four evidence-feasibility gaps: (a) NFR-021's
zero-leak and occupancy rows lean on observability surfaces (mailbox
introspection, pool drain-state, relay-backend identification across three
PG instances) that no FR requires to exist; (b) the FR-084 counter list is
missing two counters other rows rely on (depth histogram,
relay-rate-per-hop-round) and disagrees with FR-085/NFR-022 on the
state-bytes field names; (c) several drill assertions (peak backend count
at max depth, killed-terminal-node mid-drain, post-cancel zero orphans,
drain-all-local-first, handoff-target rule) need timing hooks or a relay
trace facility the specs do not yet require; (d) two normative inputs the
ACs reference are not yet pinned (direct-mode timeout semantics, the
reduced-depth informational row's parameters), so their evidence cannot be
pre-registered as written.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | NFR-021's verification methods name observability surfaces no FR requires to exist: (a) "mailbox introspection after TC-047 drills" — FR-088 specs the mailbox but no introspection function/view (e.g. `ec_distann_mailbox_status()` returning slot states) to assert "leaked mailbox slots = 0"; (b) "undrained pooled connections = 0" — neither FR-086 nor the D5 pool discipline exposes pool/busy-until-drained state to a test; (c) the pg_stat_activity backend counts must run per-instance across all 3 PG instances and need a way to identify relay backends (no spec requires relay sessions to set a distinguishing `application_name`/session tag, and FR-079's session-identity discipline does not define one). Add the introspection surface(s) and a fixed relay application_name tag to FR-086/FR-088, and state "assert per instance, all roster nodes" in NFR-021's Method column | NFR-021, FR-086, FR-088, TC-046, TC-047 |
| FND-002 | medium | The depth histogram is not in FR-084's required counter list (`relay_hops, relay_depth_max, state_bytes_out/in, drains_executed, handoffs_per_node, fallback_resumed`), yet FR-089's behavior ("the depth histogram SHALL record relay depths reached"), FR-089-AC-5, NFR-022's counter row, and ADR-086's Measurement Requirements all rely on it; `relay_depth_max` is a scalar and cannot reproduce a histogram. Add `relay_depth_histogram` to the FR-084 counter list (and the design doc's config-surface list, which has the same omission) | FR-084, FR-089, NFR-022, ADR-086 |
| FND-003 | medium | `relay-rate-per-hop-round` — the headline D7 evidence row of NFR-022 and ADR-086's Measurement Requirements — is backed by no counter in FR-084's list and no stated derivation rule. If it is derived (e.g. `relay_hops ÷ drains_executed`, treating each drain as a hop round), pre-register the formula in NFR-022's Method column; otherwise add the counter to FR-084. Without this the D7 row cannot be computed from results.jsonl fields the specs require to exist | NFR-022, FR-084, ADR-086 |
| FND-004 | medium | State-bytes counter naming is inconsistent across the evidence chain: FR-084 registers `state_bytes_out/in`, while FR-085 ("state_bytes max and total per query"), NFR-021's envelope row, NFR-022's counter row, and ADR-086 all cite `state bytes max/total`. FR-085-AC-5 and the NFR-021 envelope assertion need the max aggregation specifically (an envelope is a per-message bound — out/in totals cannot assert it). Pin one field set (recommend `state_bytes_max`, `state_bytes_total`) in FR-084 and use it verbatim in results.jsonl | FR-084, FR-085, NFR-021, NFR-022 |
| FND-005 | medium | Three drill assertions are races as written: (a) NFR-021's "fixture drill counting backends (pg_stat_activity) at max depth" must sample at the instant the chain is at peak depth; (b) FR-088-AC-4's "terminal node is killed mid-drain" must land the kill while the drain is executing; (c) FR-087-AC-3's "no orphaned relay backends after the drill" asserts on a state reached asynchronously after cancel. The only injection GUC specced (`debug_fail_relay_depth`) injects failure, not a pause. Add a hold/stall injection point (e.g. `debug_hold_relay_depth`, NFR-020 off-by-default posture) so peak occupancy and mid-drain kills are deterministic, or replace the peak-sample with a shmem high-water concurrent-backend counter; and give the zero-orphan assertions an explicit settle rule (poll pg_stat_activity per instance until quiesce, bounded) | NFR-021, FR-087, FR-088, TC-046, TC-047 |
| FND-006 | medium | FR-086-AC-2 (drain-all-local-first, "counter-asserted"), FR-086-AC-3 (handoff target = owner of best unexpanded candidate), FR-086-AC-6 (no relay head descent, "counter/inspection"), and FR-089-AC-1 ("counter vs relay trace on the fixture") all need per-drain decision visibility — which frontier candidates were local, who was chosen, whether descent ran — that the aggregate counters (`drains_executed`, `handoffs_per_node`) cannot provide, and no spec requires a relay trace facility or a head-descent counter to exist. Spec an off-by-default debug relay-trace GUC (NFR-020 taxonomy) emitting per-drain frontier-ownership/handoff decisions, and add a seed/descent counter to the FR-081/FR-084 surface for AC-6 | FR-086, FR-089, FR-081, NFR-020 |
| FND-007 | medium | FR-087-AC-1 / D9 top-k identity rests on "the deterministic multinode fixture" but no spec states the determinism source. Same-index/same-epoch removes build determinism concerns (unlike SR-005 FND-007), but relay traversal order differs by construction, so identical top-k additionally requires either tie-free distances at the k boundary on the fixture corpus or a documented distance tie-break rule — neither is stated, and neither are the fixture pins (corpus seed, BW, H, k). Add to FR-087 (or the TC-046 row): "seeded fixture corpus with distinct pairwise distances at the k boundary (or deterministic vec_id tie-break), fixed BW/H/k recorded in the drill log" | FR-087, FR-088, ADR-086, TC-046 |
| FND-008 | medium | FR-088 leaves the direct-mode wait-timeout semantics unpinned ("error vs one coordinator-mode rerun ... pinned by spec review"), yet FR-088-AC-4 and the TC-047 `relay_wait_timeout_ms` permutation row assert behavior "per the pinned timeout semantics" — evidence for the AC cannot be defined until the decision exists. Pin it in FR-088 before B2 (recommend: classified timeout error, no silent rerun, per NFR-020's correct-or-error posture) and restate the AC concretely | FR-088, NFR-020, TC-047 |
| FND-009 | medium | NFR-022's reduced-depth informational row is not well-defined enough to pre-register: "one reduced-depth setting" pins no value (depth=1? H/2?), no scales/modes, no metrics beyond "exercise the hybrid resume", and it is silent on whether the recall-parity bar applies (FR-089-AC-3 implies resumed results must still equal coordinator's, so parity should hold even for an informational row). Pin the depth value, the cell set (e.g. both batann modes at 100k), the reported fields (`fallback_resumed` rate, depth histogram, parity delta), and state explicitly that the parity bar applies but the latency result is non-gating | NFR-022, FR-089, TC-048 |
| FND-010 | low | FR-084-AC-1's "behavior with the GUC unset is byte-identical to pre-BatANN builds" is not a producible test assertion — it is a cross-commit A/B no fixture can run. The B0 exit criterion already states the testable form; reword the AC to "existing FR-081 unit/pg test suites pass unchanged over the refactor, and coordinator-mode scans report zero relay activity in the new counters" | FR-084, TC-045 |
| FND-011 | low | FR-088's "coordinator unreachable" black-hole class is not injectable on the loopback fixture: killing the coordinator instance also kills the waiting scan, and localhost cannot be partitioned (SR-005 FND-010 precedent). Node-crash (kill the terminal instance) and forward-connect-failure (kill/poison the next node's entry) are injectable and TC-047 lists them; scope the drill classes to those loopback-injectable manifestations, or note that relay-node→coordinator delivery failure is exercised via a poisoned return route on the real multi-instance harness (task-172 style) if at all | FR-088, TC-047, NFR-021 |
| FND-012 | low | B4 evidence-chain hygiene: (a) the exact results.jsonl field names for the relay counters are not enumerated anywhere (NFR-022 lists metrics prose-style; FR-084 lists GUC-surface counter names that partially disagree, see FND-002/004) — pre-register the field schema in the TC-048 packet config; (b) the design doc's B4 bundles the suite-runner extension (mode axis + counter emission) and the gate run in one milestone, but per the FR-038 convention the runner extension must land as its own commit before the packet uses it; (c) TC-048 does not restate TC-044's protocol prerequisites (NFR-017 anchors) — partially mitigated since `distinct_recall` already exists in this worktree's suite runner | NFR-022, NFR-007, TC-048, plan/design/batann-state-passing-coordination.md |
| FND-013 | low | NFR-022's Verification pre-registers "mode × scale × recall/latency/storage", but coordination mode has no on-disk effect (ADR-086: "No on-disk change"), so storage-per-mode is a null A/B axis that would triple identical storage rows. Mark storage as mode-invariant in the pre-registered table (run once per scale) so the packet does not imply a measured mode effect that cannot exist | NFR-022, ADR-086, TC-048 |

## Reconciliation (2026-07-09, post-review spec revision)

- FND-001 **RESOLVED** — FR-088 specs `ec_distann_relay_mailbox_status()`
  (operator-gated); FR-084 mandates the `application_name =
  'ec_distann_relay'` session tag; NFR-021's Method column now says
  per-instance across all roster nodes; pool-drain state is asserted via the
  eviction rule (FR-088) plus the settle-poll drill.
- FND-002 **RESOLVED** — `relay_depth_histogram` added to the FR-084
  normative counter list and the design doc.
- FND-003 **RESOLVED** — derivation pre-registered in NFR-022:
  relay-rate-per-hop-round = `relay_hops ÷ drains_executed`.
- FND-004 **RESOLVED** — field set pinned to `state_bytes_max` /
  `state_bytes_total` in FR-084 and used verbatim in FR-085/NFR-021/NFR-022.
- FND-005 **RESOLVED** — `ec_distann.debug_hold_relay_depth` added (FR-084);
  NFR-021 occupancy row uses it to pin peak depth; FR-088-AC-4 pins the kill
  window with it; zero-orphan rows carry an explicit bounded settle-poll.
- FND-006 **RESOLVED** — `ec_distann.debug_relay_trace_notice` (per-drain
  frontier-ownership/handoff/descent trace) and the `head_descents` counter
  added to FR-084; TC-046 references the trace.
- FND-007 **RESOLVED** — ADR-086 D9a and FR-087(-AC-1) pin the fixture
  conditions: seeded corpus, deterministic tie-break at the k boundary,
  fixed BW/H/k, convergence-dominant termination with `early_exit` asserted.
- FND-008 **RESOLVED** — timeout pinned in FR-088 (non-retriable classified
  error, no rerun); AC-4 and the TC-047 permutation row restated.
- FND-009 **RESOLVED** — reduced-depth row pinned in NFR-022:
  `relay_max_depth = 4`, both batann modes, 100k, parity-gated (D9b),
  latency informational; reported fields enumerated.
- FND-010 **RESOLVED** — FR-084-AC-1 reworded to the runnable
  suite-passes-unchanged form.
- FND-011 **ACCEPTED** — drill classes scoped to loopback-injectable
  manifestations (killed terminal node, forward-connect failure); the
  coordinator-unreachable black hole remains timeout-covered by design.
- FND-012 **RESOLVED/PARTIAL** — NFR-022 requires the results.jsonl relay
  field schema pre-registered in the packet config; ADR-086/NFR-022 state
  the suite-step-as-own-commit and protocol prerequisites; exact field-name
  enumeration lands with the B4 packet config as required.
- FND-013 **RESOLVED** — NFR-022 marks storage mode-invariant (once per
  scale).
