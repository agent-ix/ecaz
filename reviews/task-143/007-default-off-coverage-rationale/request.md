# Task 143 Packet 007: Default-Off Coverage Rationale

## Request

Review the Task 143 closeout update for packet 006 feedback.

This is a documentation-only decision update. The release A/B evidence remains
unchanged and approved: leaf-score-only routing is a positive candidate, and
route overfetch stays default-off.

## Feedback Addressed

Packet 006 feedback requested that the leaf-score-only default decision be
settled on its real basis:

- **Decision:** keep `ec_spire.leaf_score_only_routing` default-off for now.
- **Rationale:** the measured release A/B covers only the 2-level exact-leaf
  grid (`nlists=128` at 10k, `nlists=1024` at 50k/100k, b0, exact f32 leaf
  centroid scoring). It does not cover deeper hierarchies, larger fan-outs, or
  approximate leaf scoring where parent `path_score` may carry signal.
- **Correction:** the half-nprobe gate is now described only as a frontier
  re-anchoring bar, not the safe-to-enable/default-on bar.
- **Both-levers-on cell:** documented as out of scope for Task 143 closeout
  because isolated overfetch is dominated; Task 146 should include the combined
  cell if it revisits leaf-only promotion.

## Changed Files

- `plan/tasks/143-spire-leaf-ranking-route-overfetch.md`
- `reviews/task-143/006-leaf-ranking-decision/request.md`
- `reviews/task-143/006-leaf-ranking-decision/artifacts/manifest.md`

## Validation

- `git diff --check`
- No tests or benchmarks rerun; this is a docs-only decision update over the
  already-approved release evidence.

