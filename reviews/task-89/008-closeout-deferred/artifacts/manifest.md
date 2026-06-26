---
head_sha: b2143eaf6914560b12e092eeb1bb55bee9619c37
task_bucket: reviews/task-89
packet: reviews/task-89/008-closeout-deferred
timestamp_utc: 2026-06-26T04:47:12Z
---

# Artifact Manifest

## Scope

Task 89 deferred closeout request. This packet introduces no new benchmarks or
code changes. It maps Task 89 acceptance criteria to the existing evidence
packets and requests reviewer approval to close the task as complete-deferred.

## Evidence Inputs

- `reviews/task-89/001-ivf-tqplus-experimental-profile/`
- `reviews/task-89/003-ivf-tqplus-dbpedia-suite/`
- `reviews/task-89/004-ivf-tqplus-qjl-projected-suite/`
- `reviews/task-89/005-ivf-tqplus-insert-drift/`
- `reviews/task-89/006-ivf-tqplus-cross-corpus/`
- `reviews/task-89/007-public-shape-defer-gate/`
- `spec/adr/ADR-081-tqplus-experimental-calibration-profile.md`
- `plan/tasks/89-turboquant-tqplus-cross-am-validation.md`

## Key Closeout Facts

- ADR-081 is accepted and selects IVF-only experimental TQ+.
- IVF no-QJL DBPedia evidence is mixed, not a durable quality win.
- IVF QJL/gamma evidence shows only a small recall gain.
- Insert drift passes measured thresholds.
- Cross-corpus synthetic evidence shows systematic recall regression:
  - -0.45 pp at `nprobe=16`.
  - -2.95 pp at `nprobe=32`.
  - -5.00 pp at `nprobe=48`.
  - -7.30 pp at `nprobe=64`.
- Existing latency rows are not used as closeout evidence because TQ+ and
  baseline TurboQuant scorer implementations are not comparable yet.

## Requested Review Outcome

Reviewer approval to close Task 89 as **complete (deferred)** after accepting
the packet 007 public-shape gate.
