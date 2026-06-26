# Task 121 Packet 025 Artifact Manifest

## Packet

- Task: 121
- Packet: `reviews/task-121/025-phase3-turboquant-block-summary-decision/`
- Head SHA: `c3947d39cd742215d6c06e7b0b225a8904ce162b`
- Packet manifest written: `2026-06-26T02:30:31Z`
- Lane: local review decision, no new benchmark execution
- Status: implementation-gap decision only

## Evidence Sources

- `reviews/task-121/001-stage1-routing-screen/request.md`: Stage 1
  `storage_format=turboquant` control, recall-neutral versus RaBitQ and not
  carried into the Phase 2 route-recovery factorial.
- `reviews/task-121/005-stage1-significant-set-replan/request.md`: Phase 2
  significant-set decision excluding TurboQuant from the route factorial while
  preserving it as a compatibility/Pareto follow-up.
- `reviews/task-121/019-phase3-local-rabitq-sampled-pruning/request.md`:
  10k RaBitQ block-summary sampled-pruning pilot, explicitly leaving
  TurboQuant/default block summaries open.
- `reviews/task-121/022-phase3-local-50k-retuned-latency-pipeline/request.md`:
  50k retuned RaBitQ pipeline evidence and explicit remaining-work note.
- `reviews/task-121/024-phase3-local-100k-retuned-latency-pipeline/request.md`:
  100k retuned RaBitQ pipeline evidence and explicit remaining-work note.
- `src/am/ec_spire/build/recursive.rs`: leaf-block summary builder.
- `src/am/ec_spire/scan/candidates.rs`: leaf-block pruning selectors/scorer.

## Code Audit Notes

Build-side summary emission exists for non-RaBitQ payloads: in
`build_leaf_block_summaries`, RaBitQ receives multi-representative summaries
and a covering radius, while non-RaBitQ formats receive a single encoded mean
summary.

```text
src/am/ec_spire/build/recursive.rs:818-850
```

Scan-side block pruning is still RaBitQ-gated:

```text
src/am/ec_spire/scan/candidates.rs:1830-1833
src/am/ec_spire/scan/candidates.rs:1893-1895
src/am/ec_spire/scan/candidates.rs:1927-1929
```

The summary scorer can score payload chunks generically, but the radius bonus
is RaBitQ-only and the selectors above disable global/sample block pruning
before TurboQuant can use those summaries:

```text
src/am/ec_spire/scan/candidates.rs:2023-2028
src/am/ec_spire/scan/candidates.rs:2172-2179
```

## Decision

Do not implement TurboQuant/default block pruning inside Task 121. Record it as
an implementation gap and a separate future task if block pruning becomes
promotion-worthy.

Rationale:

- TurboQuant was recall-neutral in the Stage 1 route screen and was not a
  route-recovery lever.
- The Phase 3 RaBitQ pruning evidence shows a narrow high-nprobe compute win,
  not a broad operating-point or I/O win.
- Enabling TurboQuant pruning would require scan-policy work and a new
  10k/50k/100k A/B matrix; that is not justified by the current Pareto.
