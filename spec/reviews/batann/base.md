---
id: SR-014
title: Base Checklist Review of the ec_distann BatANN Spec Batch
type: SpecReview
analysis: base
scope: "spec/functional/index/distann/FR-084..089, spec/non-functional/NFR-021..022, spec/adr/ADR-086, spec/tests.md TC-045..048, plan/design/batann-state-passing-coordination.md"
review_set: all
---
# SR-014: Base Checklist Review — ec_distann BatANN Spec Batch

## Summary

Checklist review of the Task 173 BatANN batch (ADR-086, FR-084..FR-089,
NFR-021..NFR-022, TC-045..TC-048). ID formats, uniqueness, and link
integrity verified mechanically (all 24 relative links in the new FR/NFR
files resolve; no duplicate FR/NFR/ADR/TC ids against main or the
task-165 branch). Every AC maps to at least one TC in `spec/tests.md`;
option-permutation rows cover the three new GUCs; error paths (version
reject, fingerprint mismatch, timeout, orphan delivery, oversize payload,
invalid GUC value) and boundary values (depth 0 / 1 / default / above-H)
are enumerated. Quire validation is clean over the batch. Residual
checklist-level notes are recorded below; the substantive design
questions live in the six analysis reviews alongside this file.

## Findings

| ID | Severity | Summary | Refs |
|----|----------|---------|------|
| FND-001 | low | Coverage rules 1–6 verified: every FR-084..089 AC and NFR-021/022 measurement row maps to TC-045..048; permutation rows added for coordination_mode, relay_max_depth, relay_wait_timeout_ms; boundary and error paths enumerated | spec/tests.md |
| FND-002 | low | FR-085/FR-086/FR-089 reference `relay_state.rs` / `src/tests/ec_distann_relay.rs` as verification homes that do not exist yet (implementation lands B0/B1); acceptable for PROPOSED status, noted so the matrix is re-checked at B0 | FR-085, TC-045 |
| FND-003 | low | RESOLVED in the post-review revision — FR-084/FR-088 pin the default (10000 ms) | FR-084, FR-088 |
| FND-004 | low | No US artifact accompanies the batch (distann precedent: StR-008 covers the program without a new US; BatANN inherits StR-008 via the NFR constrains edges) — consistent with task-161, no action | ADR-086, NFR-022 |
