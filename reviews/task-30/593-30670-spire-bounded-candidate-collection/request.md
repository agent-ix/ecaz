# Review Request: SPIRE Bounded Candidate Collection

Code checkpoint: `8d49f5e5` (`Bound SPIRE candidate collection`)

## Scope

- Adds `max_candidate_rows` as a relation option and
  `ec_spire.max_candidate_rows` as a session override.
- Resolves `rerank_width = 0` to a finite candidate limit instead of leaving
  `candidate_limit = None`.
- Replaces the post-hoc full-vector candidate ranking path with a bounded
  accumulator that keeps a heap while scoring leaf and delta rows.
- Preserves vec-id dedupe replacement semantics for retained candidates and
  allows an evicted vec-id to re-enter if a later duplicate is good enough.
- Updates scan-sanity wording so full-frontier recall guidance also calls out
  the candidate-row cap.
- Marks the first two Phase 10.1 bounded-candidate items complete.

## Notes

- I did not add `max_candidate_rows` columns to
  `ec_spire_index_options_snapshot`: adding four more fields pushed that pgrx
  table-returning function past its generated tuple support. The budget is
  still available as a reloption/GUC and is enforced in scan planning.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test --no-default-features --features pg18 max_candidate_rows --lib`
- `cargo test --no-default-features --features pg18 candidate --lib`
- `cargo test --no-default-features --features pg18 test_ec_spire_options_snapshot_sql --lib`
- `cargo test --no-default-features --features pg18 scan_sanity_status_reports_empty_approximate_and_full_scan --lib`

## Review Focus

- Confirm the bounded accumulator preserves ranking and dedupe semantics.
- Confirm the `max_candidate_rows = 0` auto ceiling is acceptable as the default
  hard cap for this phase.
- Confirm deferring SQL snapshot visibility for the new cap is acceptable, or
  suggest a lower-arity diagnostic surface for Phase 10.1 diagnostics.
