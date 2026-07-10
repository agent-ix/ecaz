---
type: index
title: "Plan-002 — ec_distann BatANN state-passing coordination"
description: "Contents of the Plan-002 bundle."
okf_version: "0.1"
---
# Plan-002 — ec_distann BatANN state-passing coordination

## Contents

* [Plan-002: ec_distann BatANN state-passing coordination (B0–B4)](./plan.md) - Plan overview, dependency graph, tracks, gates, test plan.
* [Task-001: B0 — beam-state seam, relay-state serde, local relay identity](./tasks/Task-001-b0-state-seam.md) - DistannBeamState refactor, wire format, GUC surface (TC-045).
* [Task-002: B1 — stack-mode relay, cancellation enabler, kill-check gate](./tasks/Task-002-b1-stack-mode.md) - Node→node relay, depth budget + terminal resume, Gate G0.
* [Task-003: B2 — direct return: flush spike, shmem mailbox, deliver endpoint](./tasks/Task-003-b2-direct-mode.md) - Gate G1 spike, mailbox, at-most-once delivery (TC-047).
* [Task-004: B3 — cross-cutting fault matrix + resource-bound drills](./tasks/Task-004-b3-faults.md) - Fault orchestration + NFR-021 evidence.
* [Task-005: B4 — three-way coordination-mode bench gate](./tasks/Task-005-b4-bench-gate.md) - NFR-022 matrix, D7 finding, Gate G2, ADR-086 verdict.
* [Task-006: suite-runner relay extension](./tasks/Task-006-suite-runner-relay-extension.md) - Mode axis + relay-counter emission (Track B).
