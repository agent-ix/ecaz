---
id: SR-008
title: Failure-Domain Analysis of the ec_distann BatANN Spec Batch
type: SpecReview
analysis: failure-domain
scope: "spec/functional/index/distann/FR-084..089, spec/non-functional/NFR-021..022, spec/adr/ADR-086, spec/tests.md TC-045..048, plan/design/batann-state-passing-coordination.md"
review_set: all
---
# SR-008: Failure-Domain Analysis of the ec_distann BatANN Spec Batch

## Summary

The batch is unusually failure-aware for a first draft (fingerprint-per-hop, error-delivery-by-failing-node, depth-0 equivalence anchor, zero-leak drills), but the direct-return mailbox protocol has real identity and duplicate-delivery holes: query_id uniqueness is scoped only to concurrent scans (permitting reuse-collision with late deliveries), the error-delivery obligation is not scoped to before-vs-after a successful handoff (a double-delivery race), the deliver endpoint has no caller-authentication posture despite a publicly computable fingerprint, and relay state carries no structural-integrity validation beyond version+fingerprint. Stack mode is missing an orphan-chain story for link failure (as opposed to cancel), and FR-089 never says whether a depth-exhaustion resume may relay again. Findings below; none require reopening ADR-086's core decisions — they are spec-tightening items for FR-085/FR-086/FR-087/FR-088/FR-089 and NFR-021.

## Findings

| ID | Severity | Summary | Refs |
|----|----------|---------|------|
| FND-001 | high | query_id is required unique only "among concurrent scans on that host" — this explicitly permits reuse after free, so a late delivery from a black-holed/timed-out chain can match a NEW scan's slot carrying the same query_id and same (still-valid) epoch fingerprint and hand it the wrong query's results; FR-088 needs non-reusable ids (generation counter / 64-bit monotonic shmem allocator) so late deliveries can never alias a live slot | FR-088 Behavior b1; FR-088-AC-5; ADR-086 D10 |
| FND-002 | high | The "any relay node that fails SHALL itself deliver the failure" rule is not scoped to failures occurring BEFORE a successful handoff: an intermediate failing after send-and-abandon succeeded (drain of the busy connection errors, local abort, etc.) would deliver an error racing the downstream chain's success delivery for the same query_id — first-writer-wins is nondeterministic and can fail a query that succeeded; spec must forbid error delivery after a confirmed send and define behavior for a second delivery to a filled-but-not-yet-freed slot (only unknown/freed is specified) | FR-088 Behavior b4/b5; FR-088-AC-3/AC-5; ADR-086 D4 |
| FND-003 | high | Deliver-endpoint spoofing is unaddressed: `ec_distann_deliver_result` is an ordinary SQL function, the epoch fingerprint is publicly computable via the existing `ec_distann_epoch_fingerprint(index)` surface, and query_id allocation/space is unspecified — any DB user with EXECUTE on the coordinator host can forge success/error/incomplete deliveries into a waiting scan; likewise `ec_distann_relay_search` accepts arbitrary caller-crafted state; NFR-014 covers conninfo secrecy, not endpoint authorization — the batch needs an auth posture (roster-caller check, REVOKE-from-PUBLIC + dedicated role, or unguessable per-query capability token in the state) | FR-088 Description; FR-086 Behavior b6; NFR-014-AC-1..3 |
| FND-004 | high | FR-085 mandates only version-tag and fingerprint validation — no structural validation of the state itself: duplicate beam vec_ids, expanded-flag inconsistencies, negative or inflated `hop_rounds_remaining`/`relay_depth_remaining`/BW×H budget, or oversized arrays from a corrupted/tampered/buggy sender silently break the NFR-019 cap, the visited-set dedupe invariant, and the NFR-021 envelope on every downstream node; direct-mode return address (`coordinator_node_id`,`query_id`) is likewise rewritable mid-chain with no integrity check — add a bounds/consistency validation clause and AC (reject state whose budgets exceed session-derivable maxima) | FR-085 Behavior b1/b7; FR-086-AC-4; NFR-019; NFR-021 |
| FND-005 | high | Mid-handoff state loss is asymmetric and only half-specified: in stack mode a link failure (A→B connection drops while B drains — distinct from cancel/timeout, which D10 covers) leaves A unwinding an error while B's live downstream chain keeps expanding as a zombie (cancel-token propagation has no trigger on link death), violating NFR-021's zero-orphan intent; the connection-loss error class (retriable vs not, given the search may be re-runnable) is also unclassified — FR-087 needs a link-failure clause: teardown obligation for the orphaned sub-chain and an FR-079-style classification for chain-link loss | FR-087 Behavior b4; ADR-086 D10; NFR-021 Rationale; TC-046 |
| FND-006 | medium | Mailbox slot exhaustion is unspecified: the mailbox is a fixed shmem budget (slots × payload cap) but neither FR-088 nor NFR-021 says what registration does when all slots are taken — error, wait, or fall back to coordinator/stack mode; needs a pinned behavior + AC (recommend: classified resource error or transparent coordinator-mode fallback, never a wait that stacks a second unbounded queue) | FR-088 Behavior b1; NFR-021 Statement; ADR-086 D4 |
| FND-007 | medium | The open timeout question (error vs one coordinator-mode rerun) is a duplicate-execution hazard, not just UX: a black-holed chain is often slow, not dead — a rerun races the original chain's eventual delivery (compounding FND-001 if query_id is reused), doubles expansion work, and creates a third attempt class outside FR-082's two-attempt epoch-restart cap with unstated NFR-019 per-attempt accounting; if rerun is chosen, the old slot must be invalidated under a fresh non-reusable query_id and the rerun counted as a new NFR-019 attempt | FR-088 Behavior b4; FR-088-AC-4; FR-082 restart rule; NFR-019 |
| FND-008 | medium | FR-089 never says whether the depth-exhaustion resumed coordinator loop may relay again: "resume in the FR-081 hop loop" implies coordinator-only, but nothing forbids re-entering relay mode with a fresh depth budget, which would make total handoffs per attempt unbounded and break the NFR-021 occupancy statement (d ≤ relay_max_depth); pin it — resume is terminal coordinator mode, no further handoffs within the attempt | FR-089 Behavior b3; ADR-086 D6; NFR-021 Statement |
| FND-009 | medium | Direct-mode cancellation cannot reach downstream: after send-and-abandon there is no connection chain to propagate the cancel token through, so post-cancel drains run to completion on remote nodes (orphan work bounded only by BW×H, since no deadline travels in the state) and then deliver to a freed slot; this is probably acceptable but contradicts a literal reading of NFR-021's "no orphaned backends after cancel" — the spec should state that direct-mode cancel is coordinator-local, define whether a still-running remote drain counts as an orphan in TC-047, and note the residual-work bound | ADR-086 D10; FR-088 Behavior b5; NFR-021 M&E row 2; TC-047 |
| FND-010 | medium | Relay drains and in-flight relay states are invisible to the FR-082 retention gate, which is node-local (metadata `in_flight_count` + relation-lock live gate on the coordinator's index): a direct-mode state parked on an abandoned connection's queue is registered nowhere, and FR-086 never requires a drain to participate in the receiving node's retire gate for its duration — safety currently rests entirely on the per-hop fingerprint check failing closed after a retire/reclaim, which the spec should state explicitly (plus: force-retire on a remote node mid-flight maps to retriable restart, and a crashed coordinator's wedged count now under-counts work living on other nodes, extending FR-082-AC-6's blast radius) | FR-086 Behavior b1; FR-082-AC-3/AC-6; src/am/ec_distann/epoch_manifest.rs retention gate |
| FND-011 | medium | Stack mode has no bounded wait: `relay_wait_timeout_ms` is direct-mode-only, so a hung (not failed) remote hop blocks all d+1 backends and d connections indefinitely unless the operator happens to set statement_timeout; NFR-021's zero-leak drills cover cancel/timeout/error but not silent hang — FR-087 should either mandate a per-hop or whole-chain wait bound or explicitly document statement_timeout as the required operator control | FR-084 Behavior b2; FR-087 Behavior b4; NFR-021 M&E; TC-046 |
| FND-012 | medium | Terminal-delivery ambiguity is unspecified: if the terminal node's `ec_distann_deliver_result` call fails indeterminately (connection drops after the statement was sent, before the ack), the spec defines neither retry (→ duplicate delivery, needs the idempotence rule of FND-002) nor give-up (→ silent loss, converted to a coordinator timeout that feeds the FND-007 rerun hazard); FR-088 needs an explicit at-most-once-with-timeout-backstop or idempotent-retry delivery contract | FR-088 Behavior b3/b4; FR-088-AC-4 |
| FND-013 | low | Direct return assumes the coordinator is a roster member whose node_id resolves to reachable conninfo from every data node; a coordinator outside the roster, or a node_id remap across a mid-flight republish, routes delivery to the wrong host — the fingerprint check there fails closed, but the resulting loss is an undeliverable black hole (timeout), which the spec should acknowledge as the intended degradation | FR-085 Behavior b8; FR-088 Description; FR-082 publish atomicity |
| FND-014 | low | A→B→A→B ping-pong is legal within the depth budget and the spec should say so: the handoff-target rule guarantees the receiver owns the best unexpanded candidate, so every handoff yields ≥1 expansion (or terminal convergence/budget exit), bounding total handoffs by min(relay_max_depth, BW×H) — stating this per-handoff progress guarantee prevents ping-pong being misdiagnosed as a fault and justifies why no loop detection is needed | FR-086 Behavior b2/b3; FR-089 Behavior b1; ADR-086 D6/D7 |
| FND-015 | low | Send-and-abandon pool hygiene edges are unspecified: behavior when the abandoned-in-flight-send cap is hit mid-query (block, sync-ack fallback, or error), and detection/eviction of a busy-until-drained connection whose peer died (the queued result never arrives, wedging the `(conninfo,node_id)` pool key for the backend's lifetime) — NFR-021's "never leave a pooled connection undrained after scan end" needs a matching drain-deadline or evict-on-error rule | ADR-086 D4/D5; FR-088 Behavior b2; NFR-021 Statement |

## Reconciliation (2026-07-09, post-review spec revision)

All findings dispositioned in the same revision:

- FND-001 **RESOLVED** — FR-088: query_ids from a 64-bit monotonic per-host
  shmem counter, never reused within a postmaster lifetime; ADR-086 D4.
- FND-002 **RESOLVED** — FR-088/ADR-086 D4: delivery rights travel with the
  state (confirmed handoff relinquishes them; indeterminate forward delivers
  nothing); first delivery wins, duplicates dropped with WARNING.
- FND-003 **RESOLVED** — new ADR-086 D11: EXECUTE revoked from PUBLIC on
  both endpoints, roster operator role; capability tokens named as the
  escalation; FR-086/FR-088 carry the grant clauses.
- FND-004 **RESOLVED** — FR-085 structural-validation clause + FR-085-AC-6
  (bounds vs session maxima, duplicate vec_ids, flag consistency, return
  address); FR-086 validates before any index read.
- FND-005 **RESOLVED** — FR-087 link-failure clause: downstream cancel via
  retained token, FR-079 transport-error (non-retriable) class, bounded
  quiesce drilled in TC-046.
- FND-006 **RESOLVED** — FR-088: slot exhaustion → transparent
  coordinator-mode fallback, never a wait; FR-088-AC-7.
- FND-007 **RESOLVED** — FR-088: timeout is a non-retriable classified
  error, never a rerun (NFR-020 posture).
- FND-008 **RESOLVED** — FR-089/ADR-086 D6: resume is terminal coordinator
  mode; FR-089-AC-6 asserts no post-resume handoffs.
- FND-009 **RESOLVED** — ADR-086 D10 + FR-088 + NFR-021: direct-mode cancel
  is coordinator-local; bounded residual drains are not orphans provided
  they quiesce.
- FND-010 **RESOLVED** — FR-086 states relay states are invisible to the
  FR-082 retention gate and safety rests on the fingerprint failing closed;
  force-retire mid-flight → retriable restart.
- FND-011 **RESOLVED** — FR-087 documents statement_timeout as the operator
  wait bound; NFR-021 Scope carries it.
- FND-012 **RESOLVED** — FR-088: at-most-once with timeout backstop;
  indeterminate delivery outcomes deliver nothing further.
- FND-013 **ACCEPTED** — coordinator-outside-roster / node_id remap degrades
  to the fail-closed fingerprint check + wait timeout; intended degradation.
- FND-014 **RESOLVED** — FR-086 progress-guarantee bullet (ping-pong legal,
  ≥1 expansion per hop, bounded by depth budget).
- FND-015 **RESOLVED** — FR-088: cap-hit degrades to synchronous ack;
  errored/dead busy-until-drained connections are evicted, never reused.
