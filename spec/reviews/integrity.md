---
id: SR-003
type: SpecReview
analysis: integrity
scope: "spec/functional/distann/** (FR-075, FR-085; build/ FR-077, FR-078; read/ FR-079, FR-080, FR-081, FR-084, FR-086, FR-088, FR-089, FR-090; storage/ FR-076, FR-087; lifecycle/ FR-082, FR-083; index.md files); spec/non-functional/NFR-017..NFR-022; spec/adr/ADR-087 — Tasks 211-214 spec round at HEAD 8165ff2d8"
review_set: subset
title: "Integrity Analysis: elevated DistANN spec set (FR-075..FR-090 / NFR-017..022 / ADR-087)"
---
# SR-003: Integrity Analysis — Elevated DistANN Spec Set (Tasks 211-214 Round)

## Summary

Quality-gate (completeness / consistency / atomicity) analysis of the full
elevated DistANN set at HEAD `8165ff2d8`, superseding the prior FR-075..083
round of this document (retained in git history). The base checklist review
(`spec/reviews/base.md`) ran first this round; its open items FND-009/010/011
(title casing, missing Inputs/Outputs sections, uniform clause-4 citation)
and its resolved items are not re-reported here. This pass hunts deeper
integrity defects: cross-artifact contradictions, requirements with more than
one valid interpretation, missing state/attestation carriers, ACs that cannot
discriminate pass from fail, normative terms with shifting meaning, and
tier/scope ambiguity.

Net result: one high-severity cross-artifact contradiction (NFR-018's
Statement flatly forbids building the replica class that FR-084 / ADR-087 /
NFR-021 clause 4 deliberately keep as an opt-in), nine medium findings —
dominated by the Task 211-213 accelerator FRs (FR-088/FR-089/FR-090)
contradicting or lacking carriers in the unamended base FRs (FR-080, FR-081,
FR-082, FR-078, FR-075) and a `bounded` storage-class vocabulary that
NFR-021 and FR-087 define incompatibly — and five low findings. The epoch
lifecycle state machine (FR-082 + FR-087 CHECK invariants) closes over every
transition named across the set; no lifecycle hole was found. The
implementation-gap notes audited (FR-075, FR-077, FR-078, FR-079, FR-081,
FR-082, FR-083, NFR-018/019/021/022) consistently preserve their SHALL and
none waives it; the only tier-labeling defect is FND-011.

## Findings

| ID | Severity | Summary | Refs |
| ------- | -------- | -------------------------------- | ------ |
| FND-001 | high | NFR-018's Statement says a full index replica "including a derived, optional, or rebuildable performance object ... SHALL NOT be built" and names the FR-084 replica as an instance — but FR-084 (MAY hold one, opt-in GUC), FR-087 (two replica catalog relations), ADR-087 decision 2 ("remains implemented ... demoted to a non-conforming opt-in"), and NFR-021 clause 4 (non-conforming accelerator reachable via explicit opt-in) all keep it buildable; an APPROVED NFR flatly forbids what an ACCEPTED ADR ships | NFR-018, FR-084, FR-087, NFR-021, ADR-087 |
| FND-002 | medium | Head replication mandate is internally undecidable: NFR-021 clause 3 says the head SHALL be "replicated for capacity (§4.1) regardless of its capacity C" and the metric row targets "≥ 1 per roster shard", while clause 5 requires every property in the shipped default — yet FR-080 makes replicas a MAY behind `ec_distann.head_replica_count` default 0 (FR-075-AC-7), so the shipped default has zero replicas per shard; either owner-serving counts as the first copy (unstated) or clause 3/clause 5 condemn the shipped default. The metric's threshold column ("sharded ... capacity C is not a factor") silently dropping the ≥ 1 target compounds the ambiguity | NFR-021, FR-080, FR-075 |
| FND-003 | medium | FR-089 width pruning ("MAY ... fan the head search only to promising shard holders") and FR-090 fused hop ("MAY skip the dedicated FR-080 head fan-out") contradict the unamended universal SHALLs of FR-080 ("seed selection SHALL fan a head-search request to every head-shard holder") and FR-081 ("Head seeding on a multi-owner roster SHALL follow the persisted head shape: ... fans a per-shard head-search request to every head-shard holder"); neither base FR carries a crown/fusion carve-out | FR-089, FR-090, FR-080, FR-081 |
| FND-004 | medium | FR-088's precedence rule has two valid interpretations: "When an explicit `head_index_cap` is set (fixture pin) ... the build SHALL use the explicit cap" — but `head_index_cap` is a reloption with default 4096 (FR-075), so with `head_sampling_rate` > 0 and the cap at its default it is undecidable whether the law or the "explicit" cap governs; explicitly-set is not observable from the reloption value and FR-088 defines no sentinel or detection rule | FR-088, FR-075 |
| FND-005 | medium | FR-088's sizing attestation has no defined carrier: the manifest SHALL attest resolved C, rate, floor, ceiling, N, and override status "bound into the manifest digest chain", but FR-082's frozen manifest-v2 field list and the FR-078 build-options v1 (30-byte)/v2 (trained) subrecords contain no such fields, and FR-088 bumps no version and names no field — FR-088-AC-1/AC-2 are unsatisfiable against the manifest formats as specified | FR-088, FR-082, FR-078 |
| FND-006 | medium | The `bounded` storage class means different things in NFR-021 and FR-087: NFR-021 defines `bounded` as bounded by k, L, dimension, roster size, or relation/projection count — capacity C is not an admitted bounding parameter and "the head index is not on that list" — and puts the membership-only head blob under `control`; FR-087 tags four relations `bounded` with C×replica-count as the bound and tags `ec_distann_generation_head_state` (the membership-blob carrier) `bounded` rather than `control`. Since the NFR-021 conformance reader skips `bounded` rows, the class assignment is load-bearing and the two artifacts must agree | NFR-021, FR-087 |
| FND-007 | medium | Error-taxonomy contradiction on fingerprint versions: FR-079's Task-214 collapsed-granularity note states `EC_EPOCH_FINGERPRINT_VERSION` "is no longer a distinct code" (message text only), while FR-082 normatively requires "A participant SHALL reject an unknown fingerprint version as `EC_EPOCH_FINGERPRINT_VERSION`" and lists it as a distinct code in its Error Conditions table | FR-079, FR-082 |
| FND-008 | medium | FR-089-AC-5 gates on `outstanding_distribution_gap=none`, but NFR-021 declares that reporting scaffolding "dead machinery left over from the deleted known-gap allowlist; it has no spec counterpart and sanctions nothing" — an AC keyed to a field with no spec meaning cannot discriminate pass from fail | FR-089, NFR-021 |
| FND-009 | medium | FR-075's AM-surface registry omits FR-088's three new reloptions (`head_sampling_rate`, `head_cap_floor`, `head_cap_ceiling`): they appear in neither FR-075 Inputs nor the AC-2 validation surface, the same registry gap the base round fixed for `ec_distann.crown_capacity` (base FND-001) but not applied to the law reloptions | FR-075, FR-088 |
| FND-010 | medium | FR-089 is internally inconsistent about serve-time behavior: the crown is "populated lazily by bounded batch RPCs" yet "There SHALL be no serve-time remote calls" with no definition of when lazy population runs; and FR-089-AC-1's "a forced miss produces identical results one RTT slower" contradicts the width-pruning clause that without FR-090 "the crown's win is owner CPU and tail width, not the round trip itself" — in an FR-089-only build a miss costs no extra RTT, so AC-1 cannot pass/fail as written | FR-089, FR-090 |
| FND-011 | low | FR-083's tier vocabulary conflates lane with implementation status: "Tier 1 — Shipped Now" contains two clauses whose own gap notes say they are not shipped/conforming (the v5 `ambulkdelete` silent-noop hazard; the fold endpoint outside the protected class), so tier membership cannot be used to infer shipped-ness even though the Description promises "every clause below names its tier" as the shipped/not-shipped discriminator | FR-083 |
| FND-012 | medium | NFR-019's normative deepening ceiling uses an undefined term: `D = max(initial_search_bar × 64, 1024)` — `initial_search_bar` is defined by no artifact in scope (FR-075 defines `top_k` as "the convergence early-exit bar", the audit note says "effective search bar"), the same undefined-provenance defect class the base round fixed for `seed_count` (base FND-003) | NFR-019, FR-075, FR-081 |
| FND-013 | low | Non-atomic acceptance criteria bundle multiple independently falsifiable obligations into single rows: FR-075-AC-5 (four claims) and AC-6 (drop + reindex), FR-078-AC-14/AC-15/AC-16 (four to six claims each), FR-082-AC-13 (six claims) and AC-15 (four claims) — a partial failure is unreportable against one AC id | FR-075, FR-078, FR-082 |
| FND-014 | low | FR-086 never states the gateway-copy set's residency scope (per-backend vs shared across backends), though its capacity is a session GUC and its sibling FR-089 states "per-backend" explicitly; FR-086-CON-1's resident-bytes bound is per-what is therefore ambiguous (capacity × backends vs capacity) | FR-086, FR-089 |
| FND-015 | low | FR-080 still carries the stale forward note "Head sizing as a scaling law ... is Task 211 scope and will amend this requirement" although FR-088 now exists, and FR-080-CON-2 still defines C as "a reloption (`head_index_cap`) with a documented default" — under FR-088 C may be law-resolved at T2 and is then not the reloption value; the promised amendment was never applied | FR-080, FR-088 |
| FND-016 | low | FR-085's Bounded Context enumerates and its Downstream scopes only FR-075..FR-084; the new in-context requirements FR-086/FR-088/FR-089/FR-090 are absent from both, the crown and fused hop have no Domain Terms entries (gateway copy does), and FR-085-AC-1's term inventory omits them — the domain model no longer covers the set it bounds | FR-085, FR-086, FR-088, FR-089, FR-090 |

## Resolutions (same session, post-review)

- FND-001 resolved: NFR-018's exclusion narrowed to conforming/measured
  lanes with the clause-4 opt-in named (no longer contradicts
  FR-084/ADR-087).
- FND-002 resolved: NFR-021 clause 3 states the shard owner counts as the
  first serving node; metric row reworded (owner + head_replica_count
  attested replicas), reconciling the default-0 GUC with clause 5.
- FND-003 resolved: FR-080 Serving and FR-081 head-seeding carry explicit
  FR-089/FR-090 carve-outs on their every-holder SHALLs.
- FND-004 resolved: FR-088 precedence is rate-only and observable.
- FND-005 resolved: FR-088 names the build-options v3 attestation carrier.
- FND-006 resolved: NFR-021 owns the vocabulary; FR-087 cites it, re-tags
  `generation_head_state` as control, and drops its own restatement.
- FND-007 resolved: FR-079's collapse note narrowed to its own endpoints;
  FR-082 retains EC_EPOCH_FINGERPRINT_VERSION as the lifecycle code.
- FND-008 resolved: FR-089-AC-5 keyed to the NFR-021 conforming verdict
  and itemised crown bytes instead of the dead gap field.
- FND-009 resolved: FR-075 registers the three FR-088 sizing reloptions as
  specced-not-yet-implemented.
- FND-010 resolved: FR-089 population timing defined (scan open, resilient
  degrade, no serve-time fetches); AC-1 reworded without the RTT claim.
- FND-011 resolved: FR-083 Tier 1 renamed "Current Contract (legacy lane;
  two flagged nonconformances)" and the Description states tier ≠
  shipped-ness.
- FND-012 resolved: NFR-019 defines initial_search_bar (top_k or the
  smaller pushed-down LIMIT, owned by FR-081).
- FND-013 OPEN (low): non-atomic ACs in FR-075/FR-078/FR-082 — splitting
  renumbers established AC ids; deferred to a dedicated pass.
- FND-014 resolved: FR-086 states per-backend residency; CON-1 scoped.
- FND-015 resolved: FR-080's stale Task-211 note now cites FR-088; CON-2
  defines C as the FR-088-resolved capacity.
- FND-016 resolved: with SB FND-002 (same defect).
