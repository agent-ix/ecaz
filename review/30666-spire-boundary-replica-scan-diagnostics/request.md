# Review Request: SPIRE Boundary Replica Scan Diagnostics

This closes Phase 9.5 by making the boundary-replica execution contract visible
in scan placement diagnostics.

Code checkpoint: `edea321e` (`Expose SPIRE boundary replica scan diagnostics`)

## Scope

- Extends `ec_spire_index_scan_placement_snapshot(index_oid, query)` with
  primary versus boundary-replica candidate row counts.
- Reports vec-id duplicate candidates suppressed by scan dedupe, split by
  primary versus boundary-replica role.
- Reports final candidate winners after vec-id dedupe and candidate limits,
  again split by role.
- Keeps top-graph and recursive routing role-agnostic: routing selects leaf PIDs;
  primary/replica role only affects candidate scoring, dedupe, and tie-breaks.
- Records the Phase 9.1 packet reference and carries the top-graph chain I/O
  attribution follow-up into Phase 10.
- Marks Phase 9.5 complete in the Task 30 Phase 9 task file and main overview.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test --no-default-features --features pg18 collect_scan_placement_diagnostics --lib`
- `cargo test --no-default-features --features pg18 rank_routed_leaf_rows_by_ip --lib`
- `cargo test --no-default-features --features pg18 collect_quantized_routed_probe_candidates --lib`

## Review Focus

- Confirm `append_scored_candidate` reports the suppressed duplicate candidate
  regardless of whether the incoming or incumbent candidate wins.
- Check that deduped and winner diagnostics are attributed to the correct
  placement and assignment role for both leaf and delta candidates.
- Check that the SQL snapshot column ordering matches the internal row mapping.
- Confirm the Phase 9.5 task completion does not overclaim: boundary replication
  remains opt-in and product-scale performance remains gated on later
  measurement packets.
