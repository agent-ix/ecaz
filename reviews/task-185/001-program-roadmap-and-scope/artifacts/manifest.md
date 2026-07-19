# Task 185 program-roadmap manifest

- Planning head: `aaa717879`
- Task bucket / packet: `reviews/task-185/001-program-roadmap-and-scope/`
- Lane: planning and task decomposition only
- Roadmap: `plan/design/ec-distann-recall-latency-roadmap.md`
- Canonical task definitions:
  - `plan/tasks/184-ec-distann-remote-payload-materialization.md`
  - `plan/tasks/185-ec-distann-gateway-landmark-selection.md`
  - `plan/tasks/186-ec-distann-bounded-hierarchical-head.md`
  - `plan/tasks/187-ec-distann-traversal-transport.md`
  - `plan/tasks/188-ec-distann-graph-search-residual.md`
  - `plan/tasks/189-ec-distann-hybrid-distance-codec.md`
  - `plan/tasks/190-ec-distann-architecture-escalation-gate.md`
- Index: `plan/tasks/README.md`
- Production/format/query effect: none
- Tests / benchmarks: not run; no executable change

## Static audit

- `git fetch origin main`: passed before task-number allocation.
- `git ls-tree --name-only origin/main:plan/tasks` filtered for 185--190:
  no matches.
- Remote branch lookup for `task-185*` through `task-190*`: no matches.
- Canonical-file count for each task number 184--190: exactly one.
- Stable-ID counts:
  - `MAT`: 40;
  - `HEAD`: 34;
  - `TRAV`: 30;
  - `GRAPH`: 18;
  - `CODEC`: 13;
  - `ARCH`: 15;
  - `NEG`: 10.
- `git diff --cached --check`: pass before planning commit.

## Evidence lineage

- Task 182 production A/B and closeout: `reviews/task-182/006..008`.
- Task 183 attribution and STOP: `reviews/task-183/002..006`.
- NFR-017 reconciliation implementation: `6d4870b6f`.
- NFR-017 reconciliation request: Task 182 packet 008.
