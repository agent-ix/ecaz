# Review Request: Task 41 SPIRE remote-search summary guards

Code commit: `535599051bcdabfe1b1594bb9a34d4fa685fb8d8`

## Summary

This packet continues Task 41 by migrating the next SPIRE remote-search summary
and coordinator diagnostic cluster from raw relation ownership to
`AccessShareIndexRelation`.

- Migrated local heap candidate summary, coordinator result summary,
  finalization summary, coordinator gate summary, coordinator local candidates,
  coordinator local summary, and the main remote-search diagnostic.
- Kept the guard alive across both AM reads in `ec_spire_remote_search`, then
  dropped it before iterator shaping.
- Converted all AM helper calls to receive `index_relation.as_ptr()` while the
  guard is live.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: 4,461 entries.
- After: 4,447 entries.
- Net change: 14 fewer grandfathered unsafe-comment baseline entries.

## Reviewer Focus

- Confirm `ec_spire_remote_search` intentionally holds one guard across both
  `spire_remote_search_candidates` and `spire_remote_search_endpoint_identity_row`
  because both are AM reads against the same validated relation.
- Confirm every other migrated function has one AM call, then drops the guard
  before result shaping.
- Confirm no SPI, environment-variable, or libpq work was moved under a guard.

## Validation

- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/unsafe-comment-audit.log`
- `make fmt-check`
  - artifact: `artifacts/fmt-check.log`
- `git diff --check 53559905^ 53559905`
  - artifact: `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18.log`
- `bash scripts/unsafe_baseline_report.sh`
  - artifact: `artifacts/baseline-after.log`
