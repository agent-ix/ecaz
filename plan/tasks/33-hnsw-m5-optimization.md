# Task 33: HNSW M5 Optimization

Status: proposed
Owner: coder1 / runtime-index track
Priority: 3

## Goal

Use the M5 laptop for HNSW follow-up only after IVF and DiskANN have fresh
optimization passes. HNSW remains important as a reference AM and a production
fallback, but the landed Task 26 conclusion still stands: threshold-only tuning
of the current in-Postgres graph build is not the highest-value next move.

This task should focus on evidence that changes the design direction:
offline/staged bulk build, direct DSM ingestion, or a clearly bounded scan/build
hot-path win.

## Baseline Rules

- Treat Task 26/ADR-048 concurrent DSM graph assembly as the current default.
- Record launched worker count, PG18 worker headroom, M5/macOS shape, release
  build, extension SHA, corpus manifest, recall, index size, and build phase
  timings in every measurement packet.
- Keep HNSW reference rows isolated from IVF and DiskANN tables unless the
  packet explicitly measures shared-table planner behavior.

## Phase 1: M5 Reference Refresh

- Re-run a small reference sweep at 50k and any locally feasible larger corpus:
  - 1, 2, 4, and 8 requested workers;
  - enough PG18 worker-process headroom to distinguish cluster limits from
    graph scaling limits;
  - current default `ConcurrentDsm` plus the diagnostic serial-leader fallback
    only where needed for A/B context.
- Capture recall@10, build wall time, graph phase time, index size, memory HWM,
  and worker launch counts.

## Phase 2: Decide The Design Lane

Pick one of these lanes before implementation:

- **Direct DSM ingestion.** Replace the remaining shm_mq tuple-ingestion
  boundary only if profiling shows it is still material at M5 speeds and the
  fallback behavior is well specified.
- **Offline/staged bulk build.** Design a faster external or staged graph build
  followed by a short PostgreSQL publish step. This is the most plausible path
  if in-Postgres graph construction remains too slow at larger scale.
- **Scan hot-path cleanup.** Only pursue scan work if HNSW is still a meaningful
  reference winner at the target recall/latency point after IVF and DiskANN
  refresh.

## Phase 3: Candidate Slices

Recommended order:

1. **Measurement refresh.** Establish M5 worker curves before changing code.
2. **Direct-ingestion design note.** If the queue/drain path is still visible,
   write the design boundary before editing `build_parallel.rs`.
3. **Offline builder ADR.** If larger builds remain uncompetitive, add an ADR
   for the staged builder rather than incrementally complicating the current
   in-Postgres path.
4. **Apple Silicon scoring pass.** Measure current arm64 scoring dispatch and
   route broad SIMD backend work through Task 21.

## Validation

- Build changes require focused PG18 validation because worker launch, DSM
  layout, and callback behavior are correctness-sensitive.
- Recall must be measured with build timing for graph-construction changes.
- Docs-only design checkpoints do not need tests; record that explicitly.

## Stop Conditions

- Do not continue worker-threshold tuning if the M5 curve repeats the Task 26
  conclusion: worker launch scales, but graph construction remains
  fundamentally too slow.
- Do not remove shm_mq ingestion as cleanup unless the new path covers every
  required build shape or has a documented fallback.
- Do not start broad HNSW work until the IVF and DiskANN M5 tasks have produced
  their first baseline packets.
