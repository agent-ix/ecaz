---
task: 119
packet: reviews/task-119/002-sidecar-rerank-matrix-support
checkpoint_sha: 5f9120c44cd6d696745e5c165135cd8cfb2ad2a4
branch: task-119-hnsw-rabitq-coarse-rerank-profile
role: coder
date: 2026-06-25
---

# Review Request: Task 119 Required Rerank Matrix Support

## Summary

This packet adds the missing benchmark harness support for Task 119's required
second-stage rerank matrix. It does **not** claim Task 119 is complete.

The previous Task 119 M5 packet only measured `heap_f32`. The task now requires
RaBitQ 1-bit candidate frontier plus this explicit rerank matrix:

- `f32`
- `rabitq2`
- `rabitq4`
- `rabitq8`
- `turboquant_2bit`
- `turboquant_3bit`
- `turboquant_4bit`
- `turboquant_5bit`
- `turboquant_6bit`
- `turboquant_7bit`
- `turboquant_8bit`

Commit `5f9120c44` extends `ecaz bench sidecar-rerank` so that matrix can be
measured over HNSW RaBitQ candidate frontiers, and adds a standard suite config
that enumerates the required 10k/50k/100k sidecar-rerank shape.

## What Changed

- Added RaBitQ sidecar variants `rabitq2` and `rabitq4`; existing `rabitq8`
  remains available.
- Added TurboQuant sidecar variants `turboquant_2bit` through
  `turboquant_8bit`.
- Added parser/label/bit-size unit tests so the variant names remain explicit.
- Added `crates/ecaz-cli/suites/task119-hnsw-rabitq-sidecar-rerank-matrix.json`
  with one `sidecar-rerank` step for each scale: 10k, 50k, 100k.

## Validation

Artifacts live under `artifacts/`:

- `cargo-check-ecaz-cli.log`: `cargo check -p ecaz-cli` succeeded.
- `cargo-test-ecaz-cli-sidecar.log`: `cargo test -p ecaz-cli sidecar -- --nocapture`
  succeeded with `9 passed`.
- `suite-audit.log`: suite audit passed with `3 steps`.
- `suite-dry-run.log` and `suite-manifest.dry-run.json`: dry-run expanded all
  three scale steps and each command includes `f32`, `rabitq2`, `rabitq4`,
  `rabitq8`, and TurboQuant `2/3/4/5/6/7/8-bit`.

## Remaining Task 119 Work

The benchmark evidence still needs to be run. The next packet should execute
the new suite on M5 against either the existing release benchmark database or a
fresh isolated database, then report 10k/50k/100k recall, latency, sidecar
bytes/storage, and candidate behavior for every required rerank representation.
