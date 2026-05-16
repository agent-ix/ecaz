# Review Request: SPIRE Top Graph I/O Attribution

Code checkpoint: `0e14157d` (`Expose SPIRE top graph tuple attribution`)

## Scope

- Advances Phase 10.4 by splitting SQL-visible top-graph object diagnostics
  into explicit meta-tuple and segment-tuple counters.
- Keeps the existing total `object_tuple_count` and `object_segment_count`
  columns while adding `object_meta_tuple_count` and
  `object_segment_tuple_count`.
- Treats a present top graph object as one metadata tuple plus zero or more
  segment tuples, matching the relation-object chain storage shape.
- Extends the small and large top-graph SQL tests so unchained and chained
  storage both prove the attribution contract.
- Marks the Phase 10.4 chained top-graph diagnostics checklist item complete.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test --no-default-features --features pg18 test_ec_spire_top_graph_snapshot_sql --lib`
- `cargo test --no-default-features --features pg18 test_ec_spire_large_top_graph_uses_chain_storage --lib`

## Review Focus

- Confirm the meta/segment tuple split is sufficient for top-graph I/O
  attribution at the current diagnostic layer.
- Confirm the new counters are compatible with the existing total tuple and
  segment counters.
- Confirm the large top-graph test covers multi-tuple chain attribution without
  depending on fragile byte-size details.
