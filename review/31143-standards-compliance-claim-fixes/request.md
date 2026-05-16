# 31143: Standards Compliance Claim Fixes

## Scope

This packet covers commit `a2e59a42`.

Changed areas:

- `spec/tests.md`
- `spec/spec.md`
- `spec/non-functional/NFR-004-safety-and-stability.md`
- `spec/non-functional/NFR-007-benchmark-provenance.md`
- `spec/non-functional/NFR-008-scale-boundary.md`
- `spec/non-functional/NFR-009-cli-drift-and-artifact-discipline.md`
- `spec/non-functional/NFR-015-benchmark-reporting-standard.md`
- `spec/stakeholder/StR-005-multi-am-vector-search.md`
- `spec/stakeholder/StR-006-benchmark-evidence-discipline.md`
- `spec/usecase/US-001..US-011`
- `spec/usecase/US-012..US-017` relationship namespace only
- `spec/usecase/US-021-cloud-benchmark-cycle.md`
- `spec/functional/FR-028..FR-038` relationship namespace only
- `spec/functional/FR-039..FR-043` tombstone files

## What Changed

- Flipped Task 34 hardening evidence claims that lacked packet-local raw logs.
  `TC-034`, `NFR-004`, and stakeholder coverage rows now treat only the
  packeted installer/MIRAI/Flux/Rudra-family logs as completed evidence and
  mark the remaining local aggregate, sanitizer, fuzz, cargo-careful, Kani,
  Loom, Shuttle, cargo-vet, cargo-geiger, AFL, PG18 sanitizer, and SQLsmith
  lanes as gaps until packeted.
- Added `GAP-019` for unpacketed Task 34 local hardening logs and retained
  `GAP-016` for live PG18 sanitizer/SQLsmith lanes.
- Added `GAP-018` so grouped summary rows are not treated as strict
  standards-complete AC-to-TC traceability.
- Added retained supersession tombstones for `FR-039..FR-043`.
- Normalized current spec relationship targets from the old
  `ix://agent-ix/tqvector/...` namespace to `ix://agent-ix/ecaz/...`.
- Migrated `US-001..US-011` and `US-021` to structured `artifact_type: US`
  frontmatter and explicit `US-XXX-AC-N` acceptance criteria.
- Clarified the master ADR duplicate-reference rule: historical duplicate ADR
  numbers require filename or canonical topic alongside numeric ID.
- Added direct `NFR-004` relationship edges to affected FRs and clarified its
  primary reliability/safety quality scope.

## Review Focus

- Confirm unsupported Task 34 claims are now partial/gap language rather than
  completed evidence.
- Confirm `FR-039..FR-043` tombstones satisfy the IEEE 828 lifecycle gap without
  reviving superseded SPIRE behavior.
- Confirm the US frontmatter/AC migration preserved intent and removed
  normative user-story language.
- Confirm relationship namespace normalization is correct for the current Ecaz
  spec.

## Validation

- `git diff --check`
- Spec inventory script:
  - StR count 7, missing IDs `[]`
  - US count 22, missing IDs `[]`
  - FR count 60, missing IDs `[]`
  - NFR count 15, missing IDs `[]`
  - no remaining US files missing `artifact_type: US`, `relationships:`,
    As/I-want/So-that story text, or at least two `US-XXX-AC-N` headings
- `rg -n "\b(SHALL|SHALL NOT|MUST|MAY)\b" spec/usecase` returned no matches.
- `rg -n "ix://agent-ix/tqvector" spec` returned no matches.

No code tests were run. This is a spec-only compliance checkpoint.
