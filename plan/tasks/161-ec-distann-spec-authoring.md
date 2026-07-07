# Task 161: ec_distann Spec Authoring (StR-008, FR-075..083, NFR-017..020, ADR-085)

Status: in progress (2026-07-06; spec batch + test matrix + full spec-review
committed on `task-161-ec-distann-specs`; closes when the batch merges).
Owner: Agent IX. One branch: `task-161-ec-distann-specs` (worktree
`~/dev/ecaz-task161`, off origin/main).
Priority: P0 — program gate zero; Tasks 162–167 implement against these
specs.

Numbering note: tasks 141–160 are double-allocated across lanes (TQ/IVF and
RaBitQ lanes on main; SPIRE remediation 141–146 on its branches). The
ec_distann program deliberately starts at 161. Evidence citations to the
remediation lane use explicit branch + packet paths.

## Why

The SPIRE remediation program measured partitioned routing out of headroom
(0.99 distinct recall costs 35.7%/78.7% of corpus scanned at 50k/100k —
`reviews/task-144/012-release-matrix-decision/` on branch
`task-144-spire-closure-ratio-pruning`). ADR-085 decides the successor:
one global Vamana graph with hash-placed records (DistributedANN,
arXiv:2509.06046). Operator decisions 2026-07-06: name `ec_distann`,
orchestrator-pull traversal, incremental insert in committed scope.

## Goal

A reviewed, Quire-valid spec batch + test matrix + architecture design doc
that Tasks 162–167 can implement without re-deriving decisions.

## Scope

- `/specify`: StR-008; FR-075..FR-083 (`spec/functional/index/distann/`);
  NFR-017..NFR-020; ADR-085; index updates. DONE (commit 3c4a22b26).
- `/spec-matrix`: TC-037..TC-044 + permutation/boundary/edge rows in
  `spec/tests.md`. DONE (3d9efbada).
- `/spec-review` (all set): 7 SpecReview docs under `spec/reviews/` +
  consolidated fixes (epoch mutation model D10, scan-restart semantics,
  write-endpoint ownership, honest D1 arithmetic, matched-recall rule,
  min-BW×H gate row, determinism, etc.). DONE (98b40e961).
- Design doc `plan/design/distann-global-graph-architecture.md` (normative
  M0–M5 definitions) + task files 161–167. THIS COMMIT.

## Required Evidence

- Quire structural validation clean over the batch (advisory EARS warnings
  only) — verified per commit.
- Review packet for merge: `reviews/task-161/001-spec-batch/`.

## Non-Goals

- No implementation code; no bench runs (Tasks 162–167).

## Acceptance Criteria

1. Spec batch, matrix, reviews, design doc, and task files committed and
   pushed on `task-161-ec-distann-specs`.
2. All spec-review high findings resolved in the specs (verifiable by
   re-reading `spec/reviews/*.md` findings against current text).
3. Review requested per the repo coder workflow before merge.

## References

- `plan/design/distann-global-graph-architecture.md`
- `spec/adr/ADR-085-ec-distann-single-global-graph.md`
- ADR-085 evidence citations (branch+packet paths)
