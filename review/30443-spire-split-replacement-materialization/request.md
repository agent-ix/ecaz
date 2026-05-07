# Review Request: SPIRE Split Replacement Materialization

## Summary

Task 30 SPIRE Phase 2 now has a pure split materialization helper.

Changes:
- Add `SpireSplitReplacementSourceRow` and
  `SpireSplitReplacementMaterialization`.
- Add `build_split_replacement_leaf_materialization`, which trains split
  replacement centroids from selected-leaf source vectors and routes normalized
  base assignments into replacement leaf inputs in PID-plan order.
- Reject stale base PIDs, delta rows, invalid dimensions, zero vectors, and
  malformed split/PID plans before live heap-source loading is wired.
- Update the Phase 2 checklist.

## Validation

- `cargo test split_replacement --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This keeps the training and
routing contract pure so the next live scheduler slice can focus on loading
source vectors for the selected split leaf.
