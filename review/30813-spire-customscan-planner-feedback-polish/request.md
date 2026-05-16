# Review Request: SPIRE CustomScan Planner Feedback Polish

Follow-up for accepted reviewer feedback on packets `30808` and `30809`. This
is a documentation/comment slice only; no planner behavior changes.

## Scope

- Documents `remote_node_count` as the total active-epoch remote placement node
  set and `remote_available_node_count` as the planner-relevant available
  subset.
- Documents why `load_custom_scan_placement_directory` remains fail-closed for
  an empty active epoch even though the SQL eligibility wrapper normally
  returns `no_active_epoch` before calling it.
- Cross-references the executor path as the place where full identity and
  manifest validation remains required.
- Pins the CustomScan production cost-model follow-up in the Phase 11 tracker:
  coordinator routing traversal, per-remote dispatch latency by fanout, and
  bounded heap-rerank/tuple-delivery cost.
- Notes that packet `30810` made pathkey declaration true-by-construction by
  adding the vector-distance ORDER BY shape gate.

## Validation

- `cargo fmt --check`
- `git diff --check HEAD -- src/am/ec_spire/custom_scan.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`

Tests were skipped because this slice changes comments and task text only.

## Review Focus

- Check that the comments accurately reflect the CustomScan planner/executor
  validation split.
- Check that the task-file cost follow-up captures the remaining production
  costing work from the `30809` review.

## Artifacts

- `review/30813-spire-customscan-planner-feedback-polish/artifacts/manifest.md`
- `review/30813-spire-customscan-planner-feedback-polish/artifacts/cargo-fmt-check.log`
- `review/30813-spire-customscan-planner-feedback-polish/artifacts/git-diff-check.log`
