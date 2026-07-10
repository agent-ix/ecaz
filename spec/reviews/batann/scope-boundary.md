---
id: SR-013
title: Scope-Boundary Analysis of the ec_distann BatANN Spec Batch
type: SpecReview
analysis: scope-boundary
scope: "spec/functional/index/distann/FR-084..089, spec/non-functional/NFR-021..022, spec/adr/ADR-086, spec/tests.md TC-045..048, plan/design/batann-state-passing-coordination.md"
review_set: all
---
# SR-013: Scope-Boundary Analysis — ec_distann BatANN Spec Batch

## Summary

Scope-boundary review of the Task 173 batch (ADR-086, FR-084..FR-089,
NFR-021..NFR-022, TC-045..TC-048, `plan/design/batann-state-passing-coordination.md`)
against the operator directive, ADR-085 D4, and the BatANN paper concepts
the batch deliberately does not adopt.

**In-scope completeness — clean.** Each of the six operator requirements has
exactly one owning artifact with no orphans and no double allocation:

| Operator requirement | Owning artifact |
|----------------------|-----------------|
| Mode GUC, default coordinator | FR-084 (ADR-086 D1) |
| Stack return mode | FR-087 (D3) |
| Direct return mode | FR-088 (D4) |
| Full mesh | FR-086 behavior + ADR-086 D5 |
| Connection pool | ADR-086 D5, hygiene bounds in NFR-021 |
| Max-depth | FR-089 (D6) |
| Measure all modes | NFR-022 / TC-048 (three-way pre-registered matrix) |

**Boundary posture — mostly clean.** ADR-086's declared untouched surfaces
(on-disk format FR-076, placement FR-078, epoch model FR-082, build path)
are genuinely untouched by FR-084..FR-089; coordination mode is correctly
kept out of the epoch fingerprint; the degenerate boundaries the operator
cares about (single-node roster, `relay_max_depth = 0`) are owned by FR-084
with equivalence ACs; NFR-021 explicitly delegates `max_connections` sizing
to the ops-docs home rather than legislating operations in spec (correct
allocation, though the target document is dangling — FND-008). The
locality-aware-placement non-goal is the model: named, rationale'd, with a
quantified reopen trigger (D7).

**Residual boundary issues**, in descending order: the read-path-only claim
is implied but never stated, and the FR-083 delta-buffer merge seam under
relay is owned by no artifact (FND-001); two shared-path changes ride the
batch without being flagged as coordinator-mode behavior changes — the
`fetch_remote_payloads` materialization relaxation (FND-002) and the B3
transport interrupt-slicing (FND-003); and of the five non-adopted paper
concepts only locality placement is named, leaving the paper's
throughput-producing concurrency model (and the absence of a throughput
metric in NFR-022) undeclared (FND-004). Milestones B0–B4 otherwise map
1:1 onto the specced scope.

## Findings

| ID | Severity | Summary | Refs |
|----|----------|---------|------|
| FND-001 | high | Read-path-only boundary is implied, never stated, and the FR-083 delta-buffer seam under relay is unowned. FR-083's interim insert merges a coordinator-local exact-scan delta buffer "into results with same-statement visibility", but no artifact says where that merge happens when `coordination_mode` is `batann_*`: relay drains accumulate hits in the travelling state and cannot see the coordinator's delta buffer, and a direct-mode delivery or incomplete-state resume changes where "results" are assembled. If the merge is a coordinator-side post-search step (like materialization, FR-085 already pins that), one sentence in FR-084 or FR-086 pins it; today FR-084's "same FR-081 search semantics" is silent because FR-081 never mentions the delta buffer. TC-045..048 also carry no concurrent-DML-under-relay drill (FR-083-AC-6 predates the mode axis), so relayed-hits × delta-hits merge and tombstone-set-mid-relay visibility are unverified in batann modes. | FR-083, FR-084, FR-086, ADR-085 D5, spec/tests.md TC-046/TC-047 |
| FND-002 | medium | FR-087's materialization fix is a shared-path change to coordinator mode, not flagged as one. `custom_scan.rs:fetch_remote_payloads` today treats a locally-owned hit with INVALID ctid as a structural fault — a corruption detector. FR-087 relaxes it to local-directory re-resolution, and neither FR-087 nor the design doc scopes the relaxation to batann-mode scans, so B1 silently weakens the coordinator-mode structural-fault invariant (a genuinely corrupt directory entry now re-resolves or misses instead of erroring). Either gate the re-resolution on relay-produced states/modes, or state explicitly that the FR-079 structural-fault classification for this case is being redefined for all modes and update FR-079/FR-081 cross-references accordingly. | FR-087 (behavior + AC-5), plan/design/batann-state-passing-coordination.md "Materialization fix", FR-079, FR-081 |
| FND-003 | medium | Milestone B3 delivers unspecced coordinator-mode behavior: transport-wide interrupt-sliced awaits + libpq cancel propagation. The design doc concedes "Today's transport block_on lacks CHECK_FOR_INTERRUPTS" — a pre-existing coordinator-mode gap — and B3 fixes it in the shared `remote_transport.rs` used by every `ec_distann_expand_nodes` call. Cancellation behavior of coordinator-mode scans therefore changes under a batch whose ADR governs only FR-084..089/NFR-021..022; no FR in the FR-079/FR-081 family owns the new cancellability contract, and no TC asserts coordinator-mode cancel. Add the shared-path note to ADR-086 D10 (or an FR-079/NFR-020 amendment) so the B3 behavior change is traceable outside the batann family. | ADR-086 D10, plan/design/batann-state-passing-coordination.md (Hazard "Cancellation", B3 row), FR-079, NFR-020 |
| FND-004 | medium | Only one of five non-adopted paper concepts is declared out of scope. Locality-aware partitioning has the full treatment (D7: named, rationale, reopen trigger). Unnamed: (a) inter-query balancing / multiple-states-per-thread pipelining — the mechanism behind the paper's headline 1.44–2.09x *throughput*, which ADR-086 cites as motivation while NFR-022 measures latency p50/p95 only, with no QPS/throughput metric and no D7-style honesty caveat that single-query relay cannot reproduce the paper's throughput deltas; (b) replicated head index on all nodes (paper lets any node originate; here FR-086 forbids relay-node head descent but nothing says head replication is a rejected/deferred alternative); (c) per-node query-embedding caching (moot here — the query travels in the state and the quantized form is recomputed per D2 — but one line saying so closes the question); (d) ZeroMQ-style async messaging (implicitly rejected by D5 "not a new transport", never named). Add a short non-adopted-concepts block to ADR-086 Alternatives; (a) additionally warrants either a throughput row in NFR-022 or an explicit statement that throughput is out of gate scope with (a) as its reopen trigger. | ADR-086 (Context, D5, D7, Alternatives), NFR-022 |
| FND-005 | low | Beam width is pinned, silently diverging from the paper's W=64. NFR-022 pins BW/H to "the FR-081 defaults or the M4 gate settings"; relay rate, state bytes, and occupancy all scale with BW, so a single BW point can mis-state relay mode's advantage relative to the paper's operating point. Pinning is the right default for the mode A/B (mode must be the only axis in a cell), but NFR-022 should either add one informational BW-sensitivity row (mirroring the existing reduced-depth row) or state why BW sensitivity is deferred. | NFR-022 Scope, ADR-086 Context, FR-081 |
| FND-006 | low | Full mesh has no direct acceptance criterion. FR-086 states "every node can reach every other node in the roster (full mesh via shared roster)" but no AC or TC row verifies mesh reachability; it is exercised only implicitly by the 3-node fixture (where A→B→C and A→B→A drills happen to need it). A counter-asserted handoffs_per_node check that all node pairs relayed at least once on the fixture would make the operator's "fully meshed" requirement falsifiable. | FR-086, TC-046 |
| FND-007 | low | Replica / hot-standby reads are unstated for batann modes. FR-084 handles the single-node and depth-0 degenerate boundaries, but nothing states whether `ec_distann_relay_search` may execute on a standby, or whether `ec_distann_deliver_result` + the shmem mailbox are primary-only (both are function calls, likely read-compatible, but the shmem registration and roster identity on a standby are unexamined). PG17 silence is consistent with the repo's PG18-primary posture and needs no action. One sentence in FR-086/FR-088 scope would close it. | FR-086, FR-088 |
| FND-008 | low | NFR-021's sizing-guidance delegation points at a document that does not exist. "to be stated where the roster/transport operations posture (NFR-014 lift) is documented" is the correct spec-vs-ops allocation (formula normative in the NFR, deployment guidance in ops docs), but NFR-014 is a SPIRE artifact and its distann "lift" has no named home; the pointer is undischargeable until one exists. Name the target (or park the guidance in NFR-021 until the lift lands). | NFR-021 Scope, NFR-014 |
| FND-009 | low | B0/B1 delivery overlap on the relay endpoint. B0's exit criterion requires `ec_distann_relay_search` single-node identity (FR-086-AC-5, TC-045), but B1's Delivers column claims the endpoint. Harmless sequencing wrinkle; tighten the B0 row to "endpoint over the local drain, single-node only" so each deliverable has one owning milestone. | plan/design/batann-state-passing-coordination.md (milestone table), TC-045 |
| FND-010 | low | Two intentionally open decisions sit inside the boundary without a named closure artifact: direct-mode wait-timeout semantics ("error vs one coordinator-mode rerun ... pinned by spec review", FR-088) and the mailbox inline payload cap (~64 KB vs DSM overflow, design-doc open item). Both are correctly scoped to B2, but FR-088-AC-4 and AC-6 reference "the pinned timeout semantics" / "the configured cap" that no artifact yet pins — record where the pin will land (ADR-086 amendment vs FR-088 revision) so B2 has a definition of done. Relatedly, if the B2 spike falls back to direct-lite, NFR-022's `batann_direct` cells measure a different mechanism than D4 describes; the gate packet must record which variant ran ("measure all modes" honesty). | FR-088, ADR-086 D4, NFR-022, plan/design/batann-state-passing-coordination.md (Open items) |

## Reconciliation (2026-07-09, post-review spec revision)

- FND-001 **RESOLVED** — FR-084 pins the FR-083 delta-buffer merge as a
  coordinator-side post-search step in every mode (relay drains never see
  the delta buffer); TC-046 gains the delta-buffer-under-relay drill.
- FND-002 **RESOLVED** — FR-087 scopes the `fetch_remote_payloads`
  re-resolution to batann-mode scans only; coordinator mode keeps the
  FR-079 structural-fault classification.
- FND-003 **RESOLVED** — ADR-086 D10 names the shared-path cancellation
  change explicitly (fixes coordinator-mode uncancellability), lands as its
  own slice at B1; FR-087 restates it.
- FND-004 **RESOLVED** — ADR-086 Alternatives gains the five-item
  non-adopted-concepts block; NFR-022 declares throughput out of gate scope
  with inter-query balancing as the reopen trigger.
- FND-005 **RESOLVED** — NFR-022 Scope states the BW-sensitivity deferral
  (informational row permitted, non-gating) per ADR-086 Alternatives (e).
- FND-006 **RESOLVED** — TC-046 adds the counter-asserted full-mesh
  reachability check (all node pairs relayed on the fixture).
- FND-007 **RESOLVED** — FR-088 states direct mode is primary-only
  (standbys use coordinator/stack mode); PG17 silence stays per repo
  posture.
- FND-008 **RESOLVED (variant)** — the sizing guidance now lives in
  NFR-021's own Scope (concrete numbers at the D6 default), with the ops
  doc named as "the roster/transport operations documentation" rather than
  a dangling NFR-014-lift pointer.
- FND-009 **RESOLVED** — B0 owns the local-only endpoint; B1's Delivers
  column reworded to transport wiring for the B0 endpoint.
- FND-010 **RESOLVED** — both decisions pinned in this reconciliation
  (timeout = classified error, no rerun; inline cap sized to the computed
  NFR-021 envelope, no DSM overflow in v1); NFR-022 records the
  direct-mode variant run in the gate packet.
