# Task 97 Packet 022: Graviton 4 Closeout Runbook

This packet is closeout preparation only. It does not run CI, AWS, tests, or
benchmarks. It turns the remaining approved-AWS Task 97 work into a
packet-local runbook so the final Graviton 4 pass can be executed without
reconstructing requirements from scattered feedback.

## Changes

- Adds `artifacts/graviton4-closeout-runbook.md`.
- Records the required Task 97 Graviton 4 evidence:
  - `Isa::Sve2` on-host qjl32 SVE2 parity;
  - measured runtime vector length from the SVE `cntw` helper;
  - real NEON parity execution through packet 020's forced hook;
  - direct `[block-kernel-counters]` rows for
    `quant=turboquant_qjl isa=sve2` and scalar tails under `isa=scalar`;
  - IVF/SPIRE/HNSW result table shape;
  - explicit `1536d/4-bit` no-QJL structural absence note.
- Pins the snapshot/run shape to `snap-0e9c7743263e61d70` and the existing
  `ecaz bench suite` qjl32 config.

## Validation

- `git diff --check`: passed

## Not Run

- No GitHub CI.
- No AWS provisioning.
- No AWS tests or benchmarks.
- No local tests were needed because this is a documentation/runbook packet.

## Review Request

Please review whether this runbook captures the complete Task 97 Graviton 4
closeout evidence checklist and command shape, while preserving the
approval-gated boundary for AWS execution.
