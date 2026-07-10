---
id: SR-010
title: Dependency Analysis of the ec_distann BatANN Spec Batch
type: SpecReview
analysis: dependency
scope: "spec/functional/index/distann/FR-084..089, spec/non-functional/NFR-021..022, spec/adr/ADR-086, spec/tests.md TC-045..048, plan/design/batann-state-passing-coordination.md"
review_set: all
---
# SR-010: Dependency Analysis — ec_distann BatANN Spec Batch

## Summary

Dependency pass over the Task 173 BatANN batch: frontmatter `relationships:`
edges vs body `Dependencies` sections, the FR-084..FR-089 DAG vs the B0→B4
milestone order in `plan/design/batann-state-passing-coordination.md`,
hidden dependencies on unmerged lane work, enablement-vs-feature separation,
StR-008 traceability, and the deliver-endpoint auth posture under NFR-014.

**Edge existence and direction.** Every frontmatter target
(FR-075/078/079/081/082/084/085/086, StR-008) exists in this worktree, and
every `depends_on` points new→old (correct direction). However, the
frontmatter edge sets are a strict subset of the body Dependencies in five
of the six FRs and both NFRs — the divergences are itemized in FND-001/002.
Cardinalities are internally consistent with the distann family convention
(`depends_on`/`constrains` both `N:1`), though that convention is inverted
relative to the older NFR-014 (`constrains 1:N`); noted, not actionable
here (FND-009).

**DAG vs milestones.** The FR DAG supports B0→B4: FR-085 (B0) depends only
on landed FRs (FR-081/082/079); FR-084/086/087/089 (B1) depend on B0 + landed
work; FR-088 (B2) depends on B0/B1; NFR-021 (B3 drills) and NFR-022 (B4)
sit strictly downstream. No earlier milestone depends on a later one at the
requirement level. The one seam is test-matrix-level: TC-045 is marked
"Planned (B0)" but covers FR-084-AC-1..2 (GUCs) and FR-086-AC-5 (relay
endpoint), both B1 deliverables per the milestone table; B0's own exit
criterion also invokes `ec_distann_relay_search` (FND-003).

**Unmerged-work exposure.** The whole batch builds on the
`task-165-ec-distann-m3` lane, which is 112 commits ahead of `origin/main`
and carries everything the specs' reuse map names (`src/am/ec_distann/*`,
the `distann-local-multinode` suite step, the `EC_DISTANN` profile, the
Task 172 staged-corpus fixture lane, distinct_recall emission). NFR-022
additionally cites the "task-172 protocol" (task proposed, no packet has
run it) and "pipeline counters" — the `distann-pipeline` step kind from
Task 166's scope was never implemented on any branch (grep of
`crates/ecaz-cli/src/commands/bench/suite.rs` shows only `spire-pipeline`
and `distann-local-multinode`; SR-005 FND-005 flagged the same gap for the
first batch). Neither ADR-086 nor the design doc states merge
prerequisites the way `plan/tasks/166-*.md` does (FND-004).

**Enablement vs feature.** The interrupt-sliced transport awaits + cancel
propagation (ADR-086 D5/D10) fix a pre-existing coordinator-mode gap —
`src/am/ec_distann/remote_transport.rs` `block_on` sites (lines 171, 384)
have no `CHECK_FOR_INTERRUPTS`, and NFR-020's 12-case fault taxonomy has no
coordinator-cancel case — yet the requirement lives only inside FR-087's
behavior bullets and is scheduled at B3 (FND-005).

**StR coverage.** FR-084..089 reach StR-008 exactly the way FR-076..083 do:
via the `depends_on` chain to FR-075 (the only FR with an `implements`
edge; StR-008 carries the single reciprocal `satisfied_by` to FR-075),
plus the direct `constrains` edges from NFR-021/NFR-022. Pattern-consistent;
no frontmatter change needed (FND-007, concurs with SR-014 FND-004).

**Auth posture (forced open question).** NFR-014's normative content is
secret non-exposure, TLS-parameter preservation, sanitized errors, and
fail-closed schema/identity drift — it says nothing about execute-privilege
or caller authorization for node-to-node SQL endpoints, and it `constrains`
only SPIRE FRs (FR-056/057/059). FR-079 and FR-086 inherit it solely by
body prose ("posture per NFR-014"); FR-088 — which introduces
`ec_distann_deliver_result`, a cross-session write into a shmem mailbox —
does not reference NFR-014 at all (FND-006).

## Findings

| ID | Severity | Summary | Refs |
|----|----------|---------|------|
| FND-001 | medium | Frontmatter `relationships:` omit edges the body `Dependencies` declare as upstream: FR-085 omits FR-079 (index handle + mismatch semantics); FR-086 omits FR-080 (no-head-descent constraint) and NFR-014 (transport posture); FR-087 omits FR-082 (restart-once splice); FR-088 omits FR-085 (state format it consumes/delivers); FR-089 omits FR-085 (relay_depth_remaining field). Machine-readable traceability sees a thinner DAG than the prose one; add the missing `depends_on` edges | FR-085, FR-086, FR-087, FR-088, FR-089 |
| FND-002 | medium | NFR-022's frontmatter carries only FR-084 + StR-008 while its body Verification depends on NFR-017 (protocol), NFR-007 (provenance), NFR-019 and NFR-021 (per-cell counter assertions); and FR-087/FR-088/FR-089 each name NFR-022 as downstream with no reciprocal `constrains` edge in NFR-022 (its matrix directly measures stack and direct modes). Same one-way gap for NFR-021, whose body names FR-089 upstream (the resume it bounds) without a frontmatter edge. NFR→NFR omission matches the NFR-017/019/020 house style, but NFR-007/NFR-017 are load-bearing protocol dependencies for a bench gate and should be frontmatter edges; the FR-087..089 reciprocals should exist either way | NFR-021, NFR-022, FR-087, FR-088, FR-089, NFR-007, NFR-017 |
| FND-003 | low | Milestone/coverage seam: TC-045 is "Planned (B0)" but covers FR-084-AC-1..2 (GUC surface) and FR-086-AC-5 (single-node relay identity), whose deliverables (`coordination_mode`/`relay_max_depth` GUCs, `ec_distann_relay_search`) the milestone table assigns to B1; B0's own exit criterion also requires the relay endpoint. Either B0's deliverable list should include the local-only endpoint + GUC registration, or those AC rows move to the B1 exit set | plan/design/batann-state-passing-coordination.md, spec/tests.md TC-045, FR-084, FR-086 |
| FND-004 | high | Unstated merge prerequisites: every milestone B0..B4 builds on the unmerged `task-165-ec-distann-m3` lane (112 commits ahead of origin/main: `src/am/ec_distann/*` scan/transport/endpoint code the reuse map refactors, `distann-local-multinode` suite step, `EC_DISTANN` profile, Task 172 staged-corpus fixture lane, distinct_recall emission). B4 additionally leans on (a) the "task-172 protocol" — Task 172 is proposed with no executed packet, so NFR-022's Scope cites a protocol that exists only as a task description; and (b) "pipeline counters in results.jsonl" — the `distann-pipeline` step kind (Task 166 scope) was never implemented (suite.rs has only `spire-pipeline` and `distann-local-multinode`), so the relay-counter emission path NFR-022 pre-registers against does not exist and must be named as B4 scope against a concrete step kind. Before B4 can run: task-165 lane merged to main (or B-work explicitly declared to stay on that lane), Task 172's protocol pinned by a landed packet, and the counter-emitting step kind landed as its own commit per the FR-038 suite rule. State these in ADR-086/design doc the way `plan/tasks/166-*.md` states its "Prerequisite merges" | NFR-022, ADR-086, plan/design/batann-state-passing-coordination.md, plan/tasks/166-ec-distann-m4-bench-gate.md, plan/tasks/172-ec-distann-real-multinode-benchmark-gate.md, crates/ecaz-cli/src/commands/bench/suite.rs |
| FND-005 | medium | Enabler buried in a feature FR: interrupt-sliced transport awaits + downstream cancel propagation (ADR-086 D5/D10) fix a pre-existing coordinator-mode gap — `remote_transport.rs` `block_on` (lines 171, 384) has no `CHECK_FOR_INTERRUPTS`, so today's FR-079/FR-081 scans are uncancellable while blocked on a remote call, and NFR-020's fault taxonomy has no coordinator-cancel case — yet the requirement is stated only inside FR-087's behavior and scheduled at B3. Name it as its own enabler slice (constraining FR-079/NFR-020, landable at/before B0) so coordinator mode gets the fix regardless of the BatANN verdict and B1/B2's much longer nested awaits are not uncancellable for two milestones | FR-087, ADR-086 D10, FR-079, NFR-020, src/am/ec_distann/remote_transport.rs |
| FND-006 | high | Deliver/relay endpoint auth posture is an unowned requirement (forced open question): NFR-014 requires secret non-exposure, TLS preservation, sanitized errors, and fail-closed identity/schema drift — nothing about execute-privilege or caller authorization — and it `constrains` only FR-056/057/059 (SPIRE). FR-079/FR-086 inherit it by body prose only; FR-088 never references it. No spec states who may execute `ec_distann_relay_search` or `ec_distann_deliver_result`; as specced, any role that can connect can deliver an arbitrary state/error into another session's mailbox (query_id is small-space, the epoch fingerprint is readable by any roster participant — it attests epoch identity, not caller identity). Decide and spec the posture: EXECUTE revoked from PUBLIC + roster-identity requirement (or an explicit accepted-risk note for the loopback research fixture), carried either as an NFR-014 extension with frontmatter `constrains` edges to FR-079/FR-086/FR-088 or as a distann-specific transport-security clause | FR-086, FR-088, FR-079, NFR-014 |
| FND-007 | low | StR-008 traceability is pattern-consistent: FR-084..089 reach StR-008 via the depends_on chain to FR-075 (`implements`), exactly as FR-076..083 do (none of which carry a direct StR edge), plus NFR-021/NFR-022 `constrains` StR-008 directly and tests.md row 65 maps the batch to TC-045..048. No frontmatter change needed; concurs with SR-014 FND-004 | FR-075, StR-008, spec/tests.md |
| FND-008 | low | The design doc's milestone→task mapping (B0=174 .. B4=178) cites task numbers with no `plan/tasks/` files (index ends at 172); given the 141–160 double-allocation precedent, allocate the numbers in `plan/tasks/README.md` (with operator confirmation) before B0 starts, or mark the mapping provisional | plan/design/batann-state-passing-coordination.md, plan/tasks/README.md |
| FND-009 | low | Cardinality convention note: the distann family (NFR-017..022) writes `constrains` as `N:1` while NFR-014 writes `1:N` for the same semantic; internally consistent within the batch, but the corpus-wide inversion will bite any tooling that interprets cardinality. Harmonize when NFR-014 is next touched | NFR-021, NFR-022, NFR-014 |

## Reconciliation (2026-07-09, post-review spec revision)

- FND-001 **RESOLVED** — missing `depends_on` edges added: FR-085→FR-079,
  FR-086→FR-080 + NFR-014, FR-087→FR-082, FR-088→FR-085, FR-089→FR-085.
- FND-002 **RESOLVED** — NFR-022 frontmatter gains `constrains` edges to
  FR-087/FR-088/FR-089 and `depends_on` NFR-017/NFR-007; NFR-021 gains
  FR-089 (`constrains`) and NFR-019 (`depends_on`).
- FND-003 **RESOLVED** — B0's deliverable list now includes GUC registration
  and the local-only (no-transport) `ec_distann_relay_search`, matching
  TC-045's B0 scope.
- FND-004 **RESOLVED** — merge prerequisites stated in ADR-086 Measurement
  Requirements, NFR-022 Scope, the TC-048 status column, and the design
  doc's open items (task-165 lane residency, relay-counter suite step as its
  own commit — the `distann-pipeline` step kind does not exist yet —
  task-172 protocol packet).
- FND-005 **RESOLVED (variant)** — the cancellation enabler is named its own
  slice and pulled from B3 to **B1** (ADR-086 D10, design doc B1 row),
  explicitly flagged as a shared-path fix benefiting coordinator mode; not
  scheduled at B0 because B0 is transport-free by design.
- FND-006 **RESOLVED** — new ADR-086 D11 owns the auth posture (EXECUTE
  revoked from PUBLIC, roster operator role, capability-token escalation
  named); FR-086/FR-088 carry the grant clauses; FR-086 now carries the
  NFR-014 frontmatter edge.
- FND-007 **ACCEPTED** — no change needed (pattern-consistent).
- FND-008 **RESOLVED** — tasks 173–178 are allocated in this batch
  (operator-confirmed) with files under `plan/tasks/` and the README index
  updated; numbers verified free across all branches.
- FND-009 **ACCEPTED** — corpus-wide harmonization deferred to the next
  NFR-014 touch.
