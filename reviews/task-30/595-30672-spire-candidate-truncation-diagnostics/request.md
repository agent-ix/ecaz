# Review Request: SPIRE Candidate Truncation Diagnostics

Code checkpoint: `6b7fe5d1` (`Surface SPIRE candidate truncation diagnostics`)

## Scope

- Completes the Phase 10.1 diagnostic item for candidate rows seen, deduped,
  retained, and truncated.
- Changes the scan candidate accumulator to return an append outcome that
  distinguishes vec-id dedupe suppression from candidate-cap truncation.
- Adds per-store `truncated_candidate_row_count` diagnostics, including primary
  versus boundary-replica role splits.
- Exposes the new truncation counters through
  `ec_spire_index_scan_placement_snapshot(index_oid, query)`.
- Documents that retained candidates are reported as `candidate_winner_count`
  and candidate-cap drops are reported as `truncated_candidate_row_count`.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test --no-default-features --features pg18 collect_scan_placement_diagnostics_reports_candidate_truncation --lib`
- `cargo test --no-default-features --features pg18 test_ec_spire_scan_placement_snapshot_sql --lib`
- `cargo test --no-default-features --features pg18 candidate --lib`
- `cargo test --no-default-features --features pg18 collect_scan_placement_diagnostics --lib`

## Review Focus

- Confirm truncation is counted against the row actually dropped by the bounded
  accumulator, including evictions of previously retained rows.
- Confirm the diagnostic invariant remains readable:
  `candidate_row_count = deduped_candidate_row_count + truncated_candidate_row_count + candidate_winner_count`.
- Confirm adding three columns to the SQL placement snapshot is acceptable for
  the Phase 10 diagnostic surface.
