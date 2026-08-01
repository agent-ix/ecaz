---
id: SR-001
title: "base checklist review of the DistANN spec set (Tasks 211-214 speccing round)"
type: SpecReview
analysis: base
scope: "spec/functional/distann/** (FR-075..FR-090 incl. index.md files); spec/non-functional/NFR-017..NFR-022; spec/adr/ADR-087"
review_set: base
---

## Summary

Base checklist review of the Tasks 211-214 touched set at HEAD 633bfa319,
superseding the prior FR-075..FR-083 round of this document (retained in git
history). Automated checks are clean: zero broken relative links across the 28
scoped files, all frontmatter ids match filenames with `type` present, AC/CON
ids are duplicate-free and gap-free per file (including the FR-083 AC
renumbering note), FR-087's twenty-relation roster matches `sql/bootstrap.sql`
exactly in both directions, and no artifact cites the pre-elevation
`functional/index/distann` path. Cross-artifact checks pass for: FR-080's
sharded-head contract vs FR-086/FR-088/FR-089/FR-090 (bounds, staleness rule,
capacity independence, fallback-identity chain are mutually consistent);
FR-084's demotion posture vs ADR-087 and NFR-021/NFR-022 (GUC name, default,
never-decision-bearing all agree); FR-083's two-tier scoping (each clause's
tier is unambiguous); and the NFR-018/NFR-021 growth-row rebase (raw growth
reported-not-threshold, normalized bytes-per-owned-record ratio governs).
Findings below: three medium consistency gaps and eight low items, of which
FND-004 and FND-005 were fixed in place.

## Findings

| ID | Severity | Summary | Refs |
| ------- | -------- | -------------------------------- | ------ |
| FND-001 | medium | FR-075's session-GUC surface (Inputs and AC-7's default enumeration) omits `ec_distann.crown_capacity`, which FR-089 defines as a production session GUC (default 0); the AM-surface registry and its registration AC will not cover the crown when it lands | FR-075, FR-089 |
| FND-002 | medium | "head sample" term drift: FR-082's manifest table still describes `head_sample_digest` as "canonical coordinator head-sample identity" without the sharded-membership qualification FR-078's build-spec row carries (deferring to FR-080's persistence clause), and FR-082's Inputs / Building-generation / Retired-retention clauses still say "head sample" where FR-080/FR-085 make membership-only the multi-owner shape | FR-082, FR-078, FR-080, FR-085 |
| FND-003 | medium | `seed_count` is used normatively (FR-080 serving/merge/CON-1, FR-081 head seeding, FR-085 Domain Rule 9) but is defined nowhere: no reloption, GUC, default value, or owning requirement states its provenance or bound | FR-080, FR-081, FR-085, FR-075 |
| FND-004 | low | FIXED IN PLACE: `read/index.md` still labeled FR-080 "Coordinator Head Index", contradicting its post-ADR-087 title "DistANN Sharded Head Index"; label updated | FR-080, ADR-087 |
| FND-005 | low | FIXED IN PLACE: FR-086 and FR-088 Downstream cited "Task 212 crown cache" / "Task 213 fused head hop" by task number although FR-089/FR-090 now exist; converted to FR-id links | FR-086, FR-088, FR-089, FR-090 |
| FND-006 | low | FR-088 names no stable error code for the trained-policy/law-resolved-C conflict (AC-4 says only "a stable error"; FR-078 defines `EC_HEAD_TRAINING` and `EC_BUILD_ID_CONFLICT` — which applies is unstated), and the `head_sampling_rate` reloption default value is unstated (implied 0 via "shipped default SHALL remain the explicit cap") | FR-088, FR-078 |
| FND-007 | low | FR-087's `ec_distann_head_shard_replica` section cites "[FR-080] §4.1 semantics"; §4.1 is a DISTRIBUTEDANN-paper section (as NFR-021's rationale establishes), not a section of FR-080, so the reference reads as a dangling internal pointer without that context | FR-087, FR-080, NFR-021 |
| FND-008 | low | Head-search endpoint naming is unowned: FR-080's flows use generic `head_search(...)` while FR-087 names `ec_distann_head_search_physical` as the replica-routing reader — a function FR-080 never defines; the export/import/populate names are consistent, the search entry point is not | FR-080, FR-087 |
| FND-009 | low | Title casing drifts between "Distann" (FR-076..FR-079, FR-081..FR-084, NFR-017..NFR-021 titles) and "DistANN" (FR-080, FR-085..FR-090, index labels); cosmetic but inconsistent with FR-085's domain-model branding | FR-076..FR-090 |
| FND-010 | low | FR-080, FR-086, FR-089, and FR-090 omit the base checklist's Inputs/Outputs sections (behavior-only style); FR-080 in particular defines its GUCs, caps, and wire behaviors inline without an input inventory | FR-080, FR-086, FR-089, FR-090 |
| FND-011 | low | FR-084 and FR-075 cite the replica as non-conforming "under NFR-021 clause 4", but the substantive violation is clauses 1-2 (coordinator-resident unsharded O(N) state); clause 4 only governs opt-in reachability/labeling — the citation is uniform across artifacts, so this is an observation, not a contradiction | FR-084, FR-075, NFR-021 |

## Resolutions (same session, post-review)

- FND-001 resolved: FR-075's head-topology GUC bullet now registers
  `ec_distann.crown_capacity` as specced-not-yet-implemented (FR-089).
- FND-002 resolved: FR-082's `head_sample_digest` manifest row now carries
  the sharded-membership qualification, deferring the shape contract to
  FR-080's persistence clause.
- FND-003 resolved: FR-080's serving clause now defines `seed_count` as
  fixed internal policy `max(2 × BW, 32)` (benchmark override
  compile-gated).
- FND-006 resolved: FR-088 names `EC_HEAD_TRAINING` for the trained-policy
  conflict and states `head_sampling_rate` default 0.
- FND-007 resolved: FR-087 now cites the DISTRIBUTEDANN-paper §4.1
  explicitly instead of a dangling FR-080 section pointer.
- FND-008 resolved: FR-080's serving clause names
  `ec_distann_head_search_physical`.
- FND-009/010/011 remain open as low-severity style items for a later
  verbosity/consistency pass (recorded, not churned this round).
