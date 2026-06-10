# Task 94 Packet 027: Graviton 4 Closeout Runbook

This packet is closeout preparation only. It does not run CI, AWS, tests, or
benchmarks. It turns the remaining approved-AWS Task 94 work into a
packet-local runbook so the final Graviton 4 pass can be executed without
reconstructing requirements from scattered feedback.

## Changes

- Adds `artifacts/graviton4-closeout-runbook.md`.
- Records the required Task 94 Graviton 4 evidence:
  - `Isa::Sve2` on-host grouped-PQ SVE2 parity;
  - measured runtime vector length from the SVE `cntw` helper;
  - real NEON parity execution through the existing forced hook;
  - direct `[block-kernel-counters]` rows for `quant=grouped_pq isa=sve2`
    and scalar tails under `isa=scalar`;
  - IVF/DiskANN closeout matrix shape;
  - packet 026 pruning-vs-throughput and GUC-default interpretation.
- Pins the restore/run shape to `snap-0e9c7743263e61d70`, an explicitly
  approved Graviton 4 profile placeholder, and `ecaz bench suite`.

## Validation

- `git diff --check`: passed

## Not Run

- No GitHub CI.
- No AWS provisioning.
- No AWS tests or benchmarks.
- No local tests were needed because this is a documentation/runbook packet.

## Review Request

Please review whether this runbook captures the complete Task 94 Graviton 4
closeout evidence checklist and command shape, while preserving the
approval-gated boundary for AWS execution.
