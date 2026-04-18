# Review Request: DiskANN (Vamana) Task 17 Plan

Branch: `adr034-diskann-access-method`

Scope:
- `plan/tasks/17-diskann-access-method.md`

## What this slice is

Planning-only packet for ADR-034. This file is the task execution
vehicle for adding `tqdiskann` as a second index access method
alongside `tqhnsw`. No code ships under this slice — subsequent
phases 1–6 (separate review packets 11005–11010) will carry the
actual implementation.

## What changed

- Added `plan/tasks/17-diskann-access-method.md` with phased
  subtasks:
  - Phase 0 — planning (this packet plus companion review packets
    11002–11004)
  - Phase 1 — AM skeleton, reloption set, metadata page struct
  - Phase 2 — build pipeline (Vamana α-pruning on PqFastScan codes)
  - Phase 3 — scan
  - Phase 4 — insert (gated on ADR-041 acceptance)
  - Phase 5 — vacuum (gated on ADR-042 acceptance)
  - Phase 6 — cost model and planner opt-in

- Architectural constraints explicitly carried forward from ADR-034
  context: new AM lives under `src/am/diskann/`, PqFastScan kernel
  consumed as-is, new `INDEX_FORMAT_V3_DISKANN` wire tag, lock
  ordering ADRs are prerequisites not postscripts.

- Task 17 does not flip the default AM, does not alter TurboQuant,
  and does not enable SPANN or AQ/RVQ work (all out-of-scope per
  task-17 scope section).

## Review focus

- Is the phase split sensible? Specifically: does gating phase 4
  on ADR-041 and phase 5 on ADR-042 make the critical path
  reviewable, or does splitting insert/vacuum into separate
  ADRs risk drift between the two lock-ordering rules that ought
  to share invariants?
- Is the scope boundary around shared PqFastScan code tight
  enough? The task prohibits in-place-editing shared helpers from
  inside `src/am/diskann/` and requires a tracked refactor
  subtask first; does that rule match how ADR-012 and task 15
  intend shared-seam evolution to work?
- Is the `tqdiskann` reloption set (`graph_degree`,
  `build_list_size`, `alpha`, `storage_format`) right for a v0
  AM, or should any of those be GUC-first instead of
  reloption-first? ADR-016 argues reloption is authoritative with
  GUC as override for `tqhnsw`.
- Is the review-packet numbering plan (11001–11010) consistent
  with how coder2 uses the 10000-range?

## Questions to answer

- ~~Should the Vamana reloption `alpha` be a rational or a float?~~
  Resolved 2026-04-18: `f32`, pgvectorscale-compatible. Rational
  buys no determinism we don't already have (f32 scoring path
  dominates), and the two-field reloption surface is worse UX.
- Phase 6 lifts the ADR-011-style planner gate for `tqdiskann`
  independently of whether the `tqhnsw` gate has lifted. Is that
  coherent, or should both AMs move through the gate together?
- Is "Recall@10 ≥ 0.90 at default tuning on the real 10k fixture"
  the right phase-3 exit gate, or should it match the
  FR-010-AC-2 post-vacuum bar of 80% of pre-vacuum recall
  instead? The current task file asks for both but at different
  phases.

## Dependencies to land first

- None. This is a pure planning artifact.

## Companion packets

- `review/11002-adr041-vamana-insert-lock-ordering/` — ADR-041
  draft.
- `review/11003-adr042-vamana-vacuum-lock-ordering/` — ADR-042
  draft.
- `review/11004-diskann-build-algorithm-design/` —
  `plan/design/diskann-build-algorithm.md` draft.

The task file cross-references these as prerequisites before
phases 2, 4, and 5 can begin.

## Not doing in this task

Explicit non-scope from the task file, worth double-checking here:

- No PqFastScan scoring-kernel changes.
- No TurboQuant code-path edits.
- No default-AM flip.
- No OPQ / AQ / RVQ / LSQ / SPANN / parallel-scan work.
- No auto-upgrade from `tqhnsw` indexes to `tqdiskann`.
