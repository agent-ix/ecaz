---
id: Task-005
title: "M4 — bench gate (program gate G2)"
type: Task
status: not_started
track: A
priority: P0
relationships:
  - target: ix://agent-ix/ecaz/Task-004
    type: depends_on
  - target: ix://agent-ix/ecaz/Task-007
    type: depends_on
  - target: ix://agent-ix/ecaz/NFR-017
    type: references
  - target: ix://agent-ix/ecaz/NFR-018
    type: references
  - target: ix://agent-ix/ecaz/NFR-019
    type: references
  - target: ix://agent-ix/ecaz/TC-044
    type: verifies
---
# Task-005: M4 — bench gate (program gate G2)

## Scope

Repo task `plan/tasks/166-ec-distann-m4-bench-gate.md` (normative). The
pre-registered four-way gate matrix (ec_distann / IVF / HNSW / best-SPIRE)
at 10k/50k/100k per NFR-017's matched-recall rule; NFR-018 ratio rows;
NFR-019 min-BW×H row; informational netem run; promote/iterate/shelve
verdict written into ADR-085 status.

## Subtasks

- [ ] **Prerequisite merges.** `task-138-spire-distinct-recall-metric`
      (distinct_recall emitter) + `task-146-spire-honest-pareto-confirmation`
      (anchors) into the measuring branch; record merge SHAs in the manifest.
- [ ] **EC_DISTANN profile** in `crates/ecaz-cli/src/profiles.rs` with a
      registered default_sweep.
- [ ] **Gate matrix run.** `ecaz bench suite`, release-verified backend at
      every node (`ecaz_build_profile()` per node in the manifest);
      anchors: IVF 100k 0.9980@37.6ms, HNSW 100k 0.9795@20.4ms
      (`reviews/task-146/006-anchor-results/`, branch
      `task-146-spire-honest-pareto-confirmation`).
- [ ] **NFR-018 rows.** Multinode storage summation per scale + transient
      build peak from the epoch manifest.
- [ ] **NFR-019 row.** Cross-scale expanded-count ratio (corpus-independent
      BW×H) + heap-read == expansion count.
- [ ] **Verdict.** promote / iterate / shelve into ADR-085 status.

## Deliverables

- Gate packet `reviews/task-166/00N-*` with the pre-registered matrix,
  merge SHAs, and the ADR-085 status update.

## Notes

- Branch `task-166-ec-distann-m4`. Never cite remediation evidence from
  task numbers 141–160 by number — branch + packet path only.
- Blocked by Task-007 (the `distann-pipeline` step kind MUST be in the
  suite release-guard whitelist before any latency evidence).
