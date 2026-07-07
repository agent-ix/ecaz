---
type: index
title: "Plan-001 — ec_distann global-graph access method"
description: "Contents of the Plan-001 bundle."
okf_version: "0.1"
---
# Plan-001 — ec_distann global-graph access method

## Contents

* [Plan-001: ec_distann global-graph access method (M0–M5)](./plan.md) - Plan overview, dependency graph, tracks, gates, test plan.
* [Task-001: M0 — single-node parity + kill-check spike](./tasks/Task-001-m0-single-node-parity.md) - AM scaffold, lean record, head index, local FR-081 loop, Gate G0.
* [Task-002: M1 — sharded build + stitch](./tasks/Task-002-m1-sharded-build-stitch.md) - FR-077 closure-overlap build, TC-038/TC-039.
* [Task-003: M2 — two-node read path](./tasks/Task-003-m2-two-node-read-path.md) - FR-078/FR-079, remote hop-rounds, Gate G1.
* [Task-004: M3 — lifecycle + fault drills](./tasks/Task-004-m3-lifecycle-faults.md) - FR-082 full, FR-083 early, TC-042 drills.
* [Task-005: M4 — bench gate](./tasks/Task-005-m4-bench-gate.md) - NFR-017/018/019 gate matrix, Gate G2, ADR-085 verdict.
* [Task-006: M5 — incremental insert](./tasks/Task-006-m5-incremental-insert.md) - FR-083 full, TC-043.
* [Task-007: suite-runner distann extension](./tasks/Task-007-suite-runner-distann-extension.md) - distann-pipeline step kind, release guard, storage summation (Track B).
