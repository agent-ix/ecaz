# Task 145: SPIRE Scan/Rerank Economy at Low Probe Counts

Status: proposed (2026-07-04; remediation program task 5 of 6).
Owner: coder (to be assigned). One coder, one branch.
Priority: P2 — the latency-at-low-probe complement to Task 144's
recall-at-low-probe. Depends on 141; sequenced after 143/144 land their
operating point (measure against it).

## Why

Whatever probe count Tasks 143/144 reach, SPIRE pays more per candidate than
IVF because the recall-economy machinery was never ported:

- SPIRE rerank is f32-only and rescores the FULL frontier: the coordinator
  path (`coordinator/hierarchy_snapshots.rs:150-259`) exact-rescores all
  candidates and ignores `ec_spire.rerank_width` (GUC exists,
  `options/mod.rs:773-781`, default 0 = full frontier).
- IVF has the pluggable `RerankScorer`/`RerankPayloadCodec` family
  (`src/am/ec_ivf/rerank.rs`: f32/f16/rabitq4/rabitq8/turboquant,
  index-resident payloads, default rerank_width=50, stage2 exact rerank at
  `scan.rs:2479-2517`) and the one deployed bound-prune win
  (Cauchy–Schwarz per-candidate skip, `ec_ivf/scan.rs:2230`, +4%).
  ADR-077's batch-kernel trade (:156) is the stated reason SPIRE lacks bound
  pruning — worth re-testing at the new, smaller candidate surface.
- ADR-074 leaf-block pruning works (1M single-node: 3.67M candidates,
  p50 32 ms, recall 0.9945) but ships default-off
  (`leaf_block_rows=0`, `leaf_block_pruning_max_blocks_per_leaf=0`,
  `options/mod.rs:85-94,837,857`) and was off in every Task 139 cell.
- Leaf size is an emergent default (`corpus/nlists`, auto = sqrt clamped to
  4096, `src/am/common/training.rs:26-36`), not a format constraint: leaf-V2
  is u32-indexed and page-spanning (FR-050). Fewer-larger-leaves geometry
  (lower fixed cost per Task 142, sub-linear within-leaf scan via block
  pruning) has never been tested because flat-scan made it unthinkable.

## Goal

At the 143/144 operating point, cut per-query scan+rerank cost so SPIRE's
p50 approaches IVF's at matched distinct recall (release IVF anchor:
0.9980 @ 37.7 ms at 100k).

## Scope

### Phase 0 — Honor rerank_width + block pruning on

- Make the coordinator path honor `rerank_width`; enable
  `leaf_block_rows`/`leaf_block_pruning_*` on the bench shape; A/B each
  alone (per-change attribution).

### Phase 1 — Port IVF rerank codec family

- Bring `rerank_format` payload codecs + stage2 exact rerank to SPIRE leaves
  (the code survey says the IVF codec is structurally portable). Sweep
  rerank_width at the low-probe operating point.

### Phase 2 — Bound pruning re-test + large-leaf geometry

- Port `posting_bound_prune` to the SPIRE leaf scan and re-test ADR-077's
  batch-vs-prune trade at the new candidate surface.
- One fewer-larger-leaves cell (e.g. nlists 128 at 100k WITH block pruning
  on + ratio probing) vs the fine-nlists frontier point.

### Phase 3 — Decision

- Promote the winning economy configuration into the Task 146 confirmation
  shape.

## Required Evidence

- `ecaz bench suite`, release build, 10k/50k/100k A/B per change (no
  stacked-aggregate benching); storage step for payload formats.

## Non-Goals

- No routing/assignment changes (143/144). No sound-pruning revival beyond
  the per-candidate bound skip (Task 131 boundary respected).

## Acceptance Criteria

1. rerank_width honored end-to-end; block pruning exercised with nonzero
   `leaf_block_*` counters in evidence.
2. Rerank codec + bound-prune ports A/B'd individually at 10k/50k/100k.
3. Large-leaf geometry cell measured; keep-or-drop decision with numbers.

## References

- `src/am/ec_ivf/rerank.rs`, `src/am/ec_ivf/scan.rs:2230,2479-2517`
- `src/am/ec_spire/scan/candidates.rs:1949-2001`,
  `coordinator/hierarchy_snapshots.rs:150-259`, `options/mod.rs:85-94,773-781`
- `spec/adr/ADR-074-*.md`, `ADR-077-*.md`,
  `spec/functional/spire/storage/FR-050-spire-leaf-v2-format.md`
- `plan/tasks/122-tq-performance-rerank-pipeline.md`,
  `124-ivf-tq-stage2-rerank-pipeline.md` (the IVF machinery's provenance)
