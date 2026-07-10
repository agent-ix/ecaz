---
id: SR-009
title: Integrity Analysis of the ec_distann BatANN Spec Batch
type: SpecReview
analysis: integrity
scope: "spec/functional/index/distann/FR-084..089, spec/non-functional/NFR-021..022, spec/adr/ADR-086, spec/tests.md TC-045..048, plan/design/batann-state-passing-coordination.md"
review_set: all
---
# SR-009: Integrity Analysis — ec_distann BatANN Spec Batch

## Summary

Requirement-language coherence, internal consistency, atomicity, and
completeness review of the Task 173 BatANN batch (ADR-086, FR-084..FR-089,
NFR-021..NFR-022, TC-045..TC-048 + coverage/permutation rows, and the
companion design doc), compared against the established distann conventions
in FR-081/FR-079/NFR-019 and ADR-085.

**Verified consistent** across all artifacts: the mode value spellings
(`coordinator` | `batann_stack` | `batann_direct`) are identical in ADR-086
D1, FR-084, NFR-022, the design doc, and the tests.md permutation rows; the
`relay_max_depth` default (= effective hop-round budget H, provisional until
B4) is stated identically in ADR-086 D6, FR-084, FR-089, and the design doc
("effective hop_rounds" ≡ H); the local-drain definition (expand all
locally-owned top-BW candidates, hand off only when the entire top-BW
frontier is remote, to the owner of the best unexpanded candidate) is
word-for-word equivalent in FR-086 and the design doc; ADR-086's D1–D10 all
exist, are internally consistent, and every D cited by an FR (D1/D6, D2/D8,
D5/D8, D3/D9/D10, D4/D10, D6/D8) says what the citing FR claims; the ADR-085
D4 reopen trigger is quoted consistently; all relative links in the eight new
FR/NFR files resolve; `relay_depth_remaining` (state field) vs
`relay_max_depth` (GUC) are used correctly everywhere; the depth-0 ≡
coordinator equivalence is stated identically in D6, FR-084, FR-089, and the
design doc; heap_tids-never-travel and budget-travels-in-state (D8/NFR-019)
are restated without drift.

**Answer to the forced D9 question**: FR-087-AC-1 ("top-k identical" on the
deterministic multinode fixture) is **aligned** with ADR-086 D9 — D9 names
top-k identity on the fixture as the evidence form and itself mandates
"identical final top-k on the fixture is [required]", which is exactly what
the AC tests. The residual wobble is labeling, not substance: FR-087's
Behavior bullet titles the identity SHALL "Recall parity", while NFR-022's
"recall-parity bar" is the different, looser ±0.001 distinct_recall@10
criterion at bench scale (FND-008).

One high finding: FR-086's receive-time "non-zero relay budget" validation
contradicts FR-089's sender-side depth decrement for the legal final handoff
(FND-001). The remaining findings are AC/Behavior misalignments, one
open-question status contradiction, counter-taxonomy drift, and
requirement-language nits.

## Findings

| ID | Severity | Summary | Refs |
|----|----------|---------|------|
| FND-001 | high | FR-086 requires the relay endpoint to validate "that a non-zero relay budget remains, before any index read", but FR-089 decrements `relay_depth_remaining` "exactly once per handoff, on the sending side" — so the legal final handoff (sender depth 1 → 0) arrives at the receiver with a zero depth budget, which FR-086's validation as written rejects. Under FR-089 a depth-0 state at a receiver is valid (drain locally; flag `incomplete` only at the next pending handoff); FR-086 as written silently shrinks the effective budget to `relay_max_depth − 1` handoffs and leaves the rejection's error class unspecified. "Relay budget" is also ambiguous between the depth budget (FR-089) and the BW×H expansion budget (D8) — neither reading makes the receive-time non-zero check coherent. | FR-086 Behavior (validation bullet); FR-089 Behavior (bullets 1–2); ADR-086 D6/D8 |
| FND-002 | medium | Direct-mode wait-timeout semantics are deliberately unpinned ("error vs one coordinator-mode rerun … pinned by spec review", FR-088), yet FR-088-AC-4 and the TC-047 permutation row assert behavior "per the pinned timeout semantics" — unverifiable as written — and this open question is absent from the design doc's "Open items tracked to milestones", which tracks every other forced open item (flush spike, depth default, mailbox cap, deepening journeys, locality). | FR-088 Behavior (error-delivery bullet), FR-088-AC-4; tests.md line 259; design doc "Open items" |
| FND-003 | medium | Mailbox payload-cap open question is answered differently across files: the design doc lists "inline cap (~64 KB) vs DSM overflow → spec review question; B2 implements the decided form", but FR-088 already decides normatively (copy "inline up to the configured cap; oversize → error status", FR-088-AC-6) and NFR-021 fixes the shmem budget as slots × inline payload cap. Either the FR/NFR pre-empt the open question (design doc stale) or the question is open (FR-088-AC-6 premature); the openness status must match. | FR-088 Behavior + AC-6; NFR-021 Statement; design doc "Open items" |
| FND-004 | medium | FR-089's iterative-deepening bullet is normative ("CustomScan deepen-on-demand re-runs SHALL each be fresh relay journeys with a fresh depth budget, counted separately") but has no FR-089 AC and appears in no TC description (TC-046/TC-048 never mention deepening re-runs; only the design doc's open-items row does, tracked to B1). "Journey" is also an undefined term relative to "scan" (FR-081) and "attempt" (NFR-019/FR-086): whether a fresh journey resets the per-attempt BW×H accounting of FR-086-AC-4/NFR-019 is unstated. | FR-089 Behavior (last bullet); tests.md TC-046/TC-048; NFR-019; FR-086-AC-4 |
| FND-005 | medium | Relay-counter taxonomy is inconsistent: FR-084 and the design doc's config section name `state_bytes_out/in` and `relay_depth_max`; FR-085, ADR-086 Measurement Requirements, and NFR-022 name `state_bytes` max/total; the design doc's wire sketch says "depth trail"; and NFR-022/FR-089 require a depth histogram and `relay-rate-per-hop-round` that FR-084's normative counter-surface list omits — so the AC-5/NFR-022 counter assertions do not name the same set of counters. | FR-084 Behavior (counter bullet); FR-085 (serializer bullet); FR-089 (counter bullet); NFR-022 Statement/table; ADR-086 Measurement Requirements; design doc "Config surface" |
| FND-006 | medium | FR-084-AC-6 (mode dispatch covers both the amgettuple and CustomScan paths) has no named TC assertion: the coverage row maps it to TC-045/TC-046, but neither TC description mentions the amgettuple path (only the coverage-row note "dual read-path dispatch" does). The claim itself is stated consistently (FR-084 Behavior, design-doc reuse map on `collect_distann_hits`) rather than as an open question, but what mode dispatch means on FR-081's cursor-only amgettuple path in a multinode roster is defined nowhere a test can bind to. | FR-084 Behavior + AC-6; tests.md lines 163, 245–246; FR-081 (eager/amgettuple bullet) |
| FND-007 | medium | FR-084-AC-1's second clause ("behavior with the GUC unset is byte-identical to pre-BatANN builds") asserts something no Behavior bullet states and is not operationally defined (byte-identical results? plans? EXPLAIN output?) — the Behavior section only fixes the default to `coordinator`. Not atomic either: default value + regression-identity are two obligations in one AC. | FR-084-AC-1 vs FR-084 Behavior |
| FND-008 | medium | FR-087's Behavior bullet labels a top-k-identity SHALL as "Recall parity (ADR-086 D9)", but D9 distinguishes the two (recall parity = the acceptance criterion; top-k identity on the fixture = the evidence form) and NFR-022's "recall-parity bar" is the distinct ±0.001 distinct_recall@10 criterion. FR-087-AC-1 itself is aligned with D9 (it tests exactly D9's mandated evidence form); the finding is the reused label naming two different bars (fixture identity vs bench ±0.001), which invites conflating them. NFR-022's Statement compounds it: "within 0.001" (two-sided) vs the table's one-sided "≥ coordinator − 0.001". | FR-087 Behavior (last bullet) + AC-1; ADR-086 D9; NFR-022 Statement vs Measurement table |
| FND-009 | low | FR-086 "a relay endpoint MAY expand only vec_ids it owns" uses MAY (permission) for what is a prohibition; should be "SHALL NOT expand vec_ids it does not own" to be enforceable requirement language. | FR-086 Behavior (ownership bullet) |
| FND-010 | low | Frontmatter `relationships` blocks are incomplete relative to the Dependencies sections, breaking the FR-081/NFR-019 precedent of mirroring: FR-085 omits FR-079; FR-087 omits FR-082; FR-088 omits FR-085; NFR-021 omits FR-089/NFR-019; NFR-022 omits NFR-017/NFR-019/NFR-021/NFR-007. | FR-085/087/088 frontmatter; NFR-021/022 frontmatter |
| FND-011 | low | The state-size envelope formula "beam ≤ seeds + BW×H×R entries × 13 B" uses R without defining it in NFR-021 or the design doc (presumably the FR-076 graph degree); BW/H are anchored to FR-075 GUCs, R is anchored to nothing. | NFR-021 Statement; design doc wire-format sketch |
| FND-012 | low | tests.md permutation row for `ec_distann.coordination_mode` lives under TC-046 and asserts "valid modes return the same top-k" including `batann_direct`, but TC-046's requirement column excludes FR-088 (direct mode is TC-047's scope) — the evidence home for the direct-mode GUC permutation is ambiguous. | tests.md lines 246–247, 257 |
| FND-013 | low | Normative Behavior bullets with no AC/TC: FR-087 intermediate pass-through ("SHALL return … exactly the bytes returned by its downstream call (no re-drain on unwind)"); FR-085 roster-resolved conninfo ("the state never carries raw conninfo strings"); FR-084 effective-mode-recorded-in-counters on a single-node roster. Each is checkable (byte comparison / format inspection / counter inspection) but currently untraced. | FR-087, FR-085, FR-084 Behavior vs AC tables |
| FND-014 | low | Terminology drift "scan" vs "attempt": FR-081 states the dedupe invariant "in one scan", FR-086/NFR-019 state it "per attempt" — "attempt" is defined only implicitly by NFR-019's restart parenthetical (FR-082 restart resets accounting, max 2). A one-line definition (attempt = one execution between FR-082 restarts) referenced from FR-086 would close the gap; interacts with FND-004's undefined "journey". | FR-081, FR-086-AC-4, NFR-019 Measurement table |

## Reconciliation (2026-07-09, post-review spec revision)

- FND-001 **RESOLVED** — FR-086 receive-time validation reworded: fingerprint
  + version + FR-085 structural bounds only; a depth-0 state at a receiver is
  explicitly valid, depth is never a receive-time rejection; budget ambiguity
  resolved by FR-085 pinning the expansion budget as the authoritative bound.
- FND-002 **RESOLVED** — FR-088 pins timeout = non-retriable classified
  error, never a rerun; AC-4 and the TC-047 permutation row restated
  concretely; design doc Open items records the resolution.
- FND-003 **RESOLVED** — decision unified: inline cap sized against the
  computed NFR-021 envelope, oversize → delivered error, no DSM overflow in
  v1; design doc updated to match FR-088/NFR-021.
- FND-004 **RESOLVED** — FR-089 defines journey (one execution of the search
  loop; each deepening re-run is a new FR-081/NFR-019 attempt with fresh
  budgets), adds the `relay_journeys` counter and FR-089-AC-6; TC-048 carries
  it.
- FND-005 **RESOLVED** — FR-084 now carries the normative counter list
  (`relay_depth_histogram`, `state_bytes_max/total`, `drains_executed`,
  `head_descents`, `relay_journeys`, …) used verbatim by FR-085/FR-089/
  NFR-021/NFR-022/ADR-086 and the design doc; relay-rate derivation formula
  pre-registered in NFR-022.
- FND-006 **RESOLVED** — FR-084 Behavior defines dispatch on both paths via
  `collect_distann_hits` (amgettuple = eager search at rescan); TC-045
  description now names the amgettuple-path dispatch assertion.
- FND-007 **RESOLVED** — FR-084-AC-1 reworded to the runnable form (default
  is coordinator; pre-existing FR-081 suite passes unchanged over the
  refactored loop, the B0 exit criterion).
- FND-008 **RESOLVED** — ADR-086 D9 rewritten as two named bars (fixture
  identity D9a vs one-sided bench bar D9b); FR-087's bullet renamed "Fixture
  bar (D9a)"; NFR-022 Statement now one-sided, matching its table.
- FND-009 **RESOLVED** — FR-086 ownership bullet now "SHALL NOT expand
  vec_ids it does not own".
- FND-010 **RESOLVED** — missing frontmatter edges added (FR-085→FR-079,
  FR-086→FR-080/NFR-014, FR-087→FR-082, FR-088→FR-085, FR-089→FR-085,
  NFR-021→FR-089/NFR-019, NFR-022→FR-087..089/NFR-017/NFR-007).
- FND-011 **RESOLVED** — NFR-021 anchors R to the FR-076 `graph_degree`
  reloption.
- FND-012 **RESOLVED** — permutation rows split: TC-046 carries
  coordinator/stack/invalid, TC-047 carries `batann_direct`.
- FND-013 **RESOLVED/ACCEPTED** — pass-through-bytes traced in the TC-046
  permutation row; no-raw-conninfo added to TC-045's inspection list;
  single-node counter attribution is covered by FR-084-AC-5's
  every-mode counter visibility (accepted as traced).
- FND-014 **RESOLVED** — FR-086 defines "attempt" (one execution between
  FR-082 restarts, max two) and scopes the invariants per-attempt.
