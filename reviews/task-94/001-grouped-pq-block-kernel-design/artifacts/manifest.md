# Task 94 Phase 1 Design Manifest

- head SHA: `2102cd61b566cebde861598eb85f0e24729a6099`
- task bucket: `reviews/task-94/`
- packet path: `reviews/task-94/001-grouped-pq-block-kernel-design/`
- phase: Phase 1 design, no code
- lane: LUT kernel family / grouped-PQ PqFastScan
- primary references:
  - `plan/tasks/94-grouped-pq-block-kernel-family.md`
  - `spec/adr/ADR-076-universal-block-kernel-pattern.md`
  - `docs/block-kernel-development.md`
  - `src/quant/lut32/{mod,scalar,neon,sve,avx2}.rs`
  - `src/am/common/{quant_codec,candidate_batch}.rs`
  - `reviews/task-91/018-adr071-072-acceptance/feedback/2026-06-09-02-reviewer.md`
  - `reviews/task-92/014-offpath-calibration-run/feedback/2026-06-09-01-reviewer.md`
- timestamp: `2026-06-09T16:10:49Z`
- validation: design-only packet; no tests or benchmarks run

## Artifacts

- `phase1-design.md`: kernel contract, scalar/SIMD pseudocode, AM registration plan, ULP and counter contract.
- `layout-audit.md`: current grouped-PQ scorer, payload, metadata, and AM call-site audit.
- `bench-suite-emitter-plan.md`: Task 94-owned plan for the `[block-kernel-counters]` latency-suite direct-row gap.

