---
id: SR-001
title: "base checklist review of the ec_distann spec batch"
type: SpecReview
analysis: base
scope: "spec/stakeholder/StR-008, spec/functional/index/distann/FR-075..FR-083, spec/non-functional/NFR-017..NFR-020, spec/adr/ADR-085, spec/tests.md TC-037..TC-044"
review_set: all
---

## Summary

Base checklist pass over the 15-artifact ec_distann batch plus its test-matrix
extension: ID formats, AC/CON shape, six coverage rules, and cross-reference
integrity. Structure is clean (no duplicate IDs, all frontmatter relationship
targets resolve, every AC row traces to a TC, boundary/permutation/edge rows
present); findings below are quality improvements, not blockers.

## Findings

| ID      | Severity | Summary                          | Refs   |
| ------- | -------- | -------------------------------- | ------ |
| FND-001 | medium   | No user-story layer for the distann lane: StR-008 traces directly to FRs while sibling lanes carry US artifacts (US-018..US-022 for SPIRE); author a distann US set when the lane stabilizes or record the omission as deliberate | StR-008, FR-075 |
| FND-002 | low      | FR error conditions are stated qualitatively (epoch-mismatch retriable, placement error) without an enumerated error-code table; matches current repo practice but weaker than checklist ideal | FR-079, FR-082 |
| FND-003 | medium   | FR-080 sources the head sample from build-shard top layers (depends on FR-077), but milestone M0 builds a single-node head index before sharded build exists; FR-080 should state the single-shard degenerate case explicitly | FR-080, FR-077 |
| FND-004 | low      | StR-008 frontmatter carries a single satisfied_by edge to FR-075; acceptable as the family root, but adding edges to NFR-017/NFR-019 would make the gate trace explicit | StR-008 |
| FND-005 | low      | TC-043 spans two milestones (M3 tombstones, M5 incremental insert); splitting into two TCs would sharpen per-milestone exit tracking | spec/tests.md |
