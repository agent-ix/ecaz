# Task 87 Phase 7 Methodology And HNSW Outcome

Head SHA: `cbb4d388bdbac0261b446b54d8ef242da130a302f`

Packet path: `reviews/task-87/023-phase7-methodology-and-closeout/`

This note addresses the cross-cutting methodology items from
`reviews/task-87/021-spire-leaf-lut32-batching/feedback/2026-06-08-01-reviewer.md`.

## Off-Path Scoring-Share Counter Decision

Packet 021 gave two acceptable closeout paths:

1. instrument the non-batched scalar scorer with equivalent counters; or
2. reframe the gate around on-path scoring counters plus end-to-end deltas,
   explicitly acknowledging that the off path is not directly comparable.

This closeout takes option 2.

Reason: the Phase 6 plumbing baseline does not have one shared scalar scorer
entry point across SPIRE and IVF. The off path is split across AM-local loops
and per-codec helpers. Adding scalar timing there would create a second,
late-stage instrumentation change whose counter scope would still not match
the Phase 7 `CandidateBatch` scorer exactly because it would include different
per-candidate loop shape and setup costs. Task 91 is the appropriate place to
unify that codec/scorer shape if direct scalar-vs-batch scorer timing is needed
later.

Therefore the final Phase 7 claim is narrower:

- on routed TurboQuant no-QJL cells, the candidate-batch-on route reaches the
  LUT32 kernel and reports direct on-path scorer counters;
- recall is preserved off/on at every routed measured cell;
- end-to-end latency deltas are reported separately and not used as a direct
  proof of a scalar-vs-LUT32 scoring-share factor;
- cells that are not TurboQuant no-QJL LUT32 routes are annotated as such.

## HNSW Investigation Outcome

Packet 021 asked why HNSW counters were zero. The answer is that the existing
real-corpus HNSW benchmark profiles used by packets 021/022 are not
TurboQuant FullLut search-codec surfaces, so they do not enter the Task 87
LUT32 candidate-batch scorer.

Evidence:

- `artifacts/hnsw-reloptions-check.log`: first exact-name lookup returned zero
  rows because the benchmark index names include suffixes such as `_m16_idx`.
- `artifacts/hnsw-reloptions-list.log`: the matching real-corpus HNSW indexes
  are source-backed HNSW profiles with reloptions such as
  `{m=16,ef_construction=128,build_source_column=source}` or only
  `{m=16,ef_construction=128}`. They do not advertise
  `storage_format=turboquant`.
- Packet 021 and packet 022 HNSW latency probes all report
  `surface=hnsw flushes=0 candidates=0 elapsed_ms=0.000000 lut32_flushes=0
  lut32_candidates=0`.

Phase 7 HNSW stop condition:

- HNSW remains on the packet 006 structural route for this task.
- The current real-corpus HNSW profiles are not valid Phase 7 LUT32 routing
  evidence because they do not exercise the TurboQuant FullLut candidate-batch
  scorer.
- Routing HNSW through the Phase 7 LUT32 kernel would require a dedicated
  TurboQuant FullLut real-corpus HNSW surface, which is outside this closeout
  and belongs with the Task 91 codec migration/parity work.

## Packet 022 Visibility

Packet 022 is now committed and pushed as
`reviews/task-87/022-phase7-50k-100k-counter-suite/`. It contains the missing
real50k and real100k counter evidence, including suite status
`completed=19 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
