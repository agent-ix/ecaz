# Task 131 Packet 023 Artifact Manifest

- head SHA: `f721bd1bfe930c02c9ddb9af17fb64ad2cbda7a8`
- task bucket: `reviews/task-131/023-phase4-bound-metadata-decision`
- timestamp: 2026-07-01 PDT
- lane: Phase 4 bound strength and metadata decision
- measurement dependency: `reviews/task-131/022-real-scale-threshold-boundability/`

## Inputs Reviewed

- Task definition: `plan/tasks/131-spire-streaming-global-topk-pruning.md`
- Real-scale boundability packet: `reviews/task-131/022-real-scale-threshold-boundability/`
- Threshold profile implementation: `src/am/ec_spire/scan/candidates.rs`
- Leaf summary storage format: `src/am/ec_spire/storage/leaf_v2_parts.rs`
- SPIRE leaf-block summary GUCs: `src/am/ec_spire/options/mod.rs`
- Prior block-summary research index:
  - `reviews/task-79/009-leaf-v3-summary-storage/request.md`
  - `reviews/task-79/010-rabitq-leaf-block-pruning/request.md`
  - `reviews/task-79/014-rabitq-global-block-pruning/request.md`
  - `reviews/task-79/022-rabitq-global-radius-block-scoring/request.md`
  - `reviews/task-81/002-local-100k-block-summary-gate/request.md`
  - `reviews/task-120/008-phase2-rabitq-block-pruning/request.md`
  - `reviews/task-120/016-phase6-maintenance-fallback-invariants/request.md`

## Current Code Facts

- `collect_quantized_selected_leaf_threshold_profile` can only count usable
  threshold bounds when `leaf_object.summaries` is non-empty and the scorer
  payload format is `RaBitQ`.
- The threshold profile uses `score_leaf_block_summary_ip_with_context` with
  full radius weight to compare a sound block upper bound against the proposed
  global threshold.
- The production-read representative cells in packet 022 built indexes without
  `ec_spire.leaf_block_rows`, so their leaves had no persisted summaries.
- Packet 022 therefore reported:
  - `sound_bound_available_sum = 0`
  - `threshold_block_available_sum = 0`
  - `threshold_row_available_sum = 0`
  - `threshold_block_skipped_sum = 0`
  - `threshold_row_skipped_sum = 0`

## Decision

Streaming global threshold feedback should be shelved for the current default
production-read surface. It has no sound bound metadata to consume.

The only plausible near-term revival path is a separate metadata-gated A/B that
builds indexes with leaf block summaries enabled, then measures whether the
sound block upper bounds are both available and selective enough at the Task
131 representative shapes.

That revival must be treated as a new storage/metadata variant, not as a small
Phase 3 protocol patch, because it changes index build options, storage bytes,
maintenance invariants, and remote version-skew behavior.

## Minimum Revival Evidence

Before implementing a coordinator-to-worker threshold protocol:

1. Build representative local multi-instance indexes with summary metadata
   enabled, for example `leaf_block_rows=16` or another Task 79/81-supported
   candidate.
2. Run `ecaz bench suite` against at least:
   - `10k`, `50k`, `100k`
   - `n128/b4/nprobe96`
   - `n1024/b2/nprobe64`
3. Capture recall, latency, storage, and the threshold profile counters:
   - `sound_bound_available_sum`
   - `threshold_block_available_sum`
   - `threshold_block_skipped_sum`
   - `threshold_row_available_sum`
   - `threshold_row_skipped_sum`
4. Compare storage and build-time cost against the default no-summary baseline.
5. Provide maintenance/fallback invariants for:
   - insert and delete visibility
   - vacuum and stale summaries
   - split or movement of leaf contents
   - remote node version skew
   - mixed indexes with and without summaries
   - strict/degraded read semantics when summaries are missing

Until those measurements exist, a streaming threshold implementation would add
protocol complexity without a recall-safe skip condition.
