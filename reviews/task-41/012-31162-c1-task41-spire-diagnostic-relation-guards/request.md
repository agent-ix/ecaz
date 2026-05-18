# Review Request: Task 41 SPIRE diagnostic relation guards

Code commit: `5c5e2f45e8c0806caa0e64d387d7651f1a650e0b`

## Summary

This packet continues Task 41 from the survey packet by migrating a coherent
cluster of SPIRE diagnostic/readiness entrypoints in `src/lib.rs` from raw
`open_valid_ec_spire_index` / `index_close` pairs to `AccessShareIndexRelation`.

- Migrated SPIRE hierarchy and top-graph snapshots.
- Migrated remote-search fanout, target, request, readiness, and execution
  diagnostic helpers.
- Migrated the adjacent libpq request, connection, dispatch, bind, and secret
  diagnostic helpers.
- Preserved the previous lock duration by explicitly dropping each guard after
  the AM helper returns owned rows or a copied summary row.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: 4,579 entries.
- After: 4,539 entries.
- Net change: 40 fewer grandfathered unsafe-comment baseline entries.

## Reviewer Focus

- Confirm every AM helper receives `index_relation.as_ptr()` only while the
  guard is live.
- Confirm each explicit `drop(index_relation)` happens after the AM helper has
  returned owned data and before iterator/result shaping.
- Confirm no SPI or environment-variable work was moved under the relation
  guard in this slice.

## Validation

- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/unsafe-comment-audit.log`
- `make fmt-check`
  - artifact: `artifacts/fmt-check.log`
- `git diff --check`
  - artifact: `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18.log`
- `make unsafe-baseline-report`
  - artifact: `artifacts/baseline-after.log`
