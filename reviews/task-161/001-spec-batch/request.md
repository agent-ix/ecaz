# Task 161 packet 001: ec_distann spec batch — review requested

- Branch: `task-161-ec-distann-specs` (worktree `~/dev/ecaz-task161`, off
  origin/main b891c3743)
- Commits under review: 3c4a22b26 (spec batch), 3d9efbada (test matrix),
  98b40e961 (spec-review docs + consolidated fixes), 2ef5fc07b (tasks
  161–167 + design doc)
- Status: review requested (2026-07-06)

## What this is

Spec-first foundation for the ec_distann program (DistributedANN-style
fifth access method: one global Vamana graph, hash-placed self-sufficient
records, coordinator hop-round search). Successor lane to the measured
partitioned-routing rejection; operator-approved plan and decisions
(ec_distann name, orchestrator-pull, incremental insert committed)
2026-07-06.

## Contents

- `spec/stakeholder/StR-008-*`, `spec/functional/index/distann/FR-075..083`,
  `spec/non-functional/NFR-017..020`, `spec/adr/ADR-085-*` (+ index files)
- `spec/tests.md`: TC-037..TC-044, distann permutation/boundary rows,
  EC-019..EC-023
- `spec/reviews/{base,failure-domain,integrity,dependency,evidence,risk-complexity,scope-boundary}.md`
  (SR-001..SR-007, review_set=all) — all findings triaged; every high fixed
  in the specs (see 98b40e961 message for the list)
- `plan/design/distann-global-graph-architecture.md` (normative M0–M5)
- `plan/tasks/161..167-ec-distann-*.md` + README index entries

## Verification

- `quire validate` structurally clean over the batch (advisory EARS grammar
  warnings only; the DuplicateArchetype lines are module-registry noise).
- No code changes; no tests run (docs/spec checkpoint per repo policy).

## Numbering notes for the reviewer

- Task IDs 141–160 are double-allocated across lanes on main; this program
  starts at 161. ADR IDs 083/084 are reserved by the SPIRE remediation
  branches; this batch uses ADR-085.
- Remediation evidence is cited by explicit branch + packet path throughout.

## Asks

1. Review the FR chain for implementability, especially FR-082's D10
   mutation model and FR-083's write-endpoint split.
2. Confirm the NFR-017 matched-recall rule and NFR-019 min-BW×H row are the
   right gate shape before Task 162 starts.
3. Flag any spec-review finding you consider unresolved.
