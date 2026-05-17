# Review Request: Task 41 coordinator relation guards

Code commit: `2ab0f0f4378bd5d906cab4084f45cd65426b74f0`

## Summary

This packet continues Task 41 by migrating the next SPIRE coordinator/routing
cluster in `src/lib.rs` from raw `Relation` open/close pairs to
`AccessShareIndexRelation`.

- Migrated root routing and routing centroid diagnostics.
- Migrated centroid classification and coordinator insert planning.
- Migrated coordinator insert dispatch planning.
- Migrated coordinator insert tuple payload preparation, keeping the guard live
  across both AM helper calls and dropping it before SPI catalog updates.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: 4602 entries.
- After: 4590 entries.
- Net change: 12 fewer grandfathered unsafe-comment baseline entries.

## Reviewer Focus

- Confirm AM helper calls only receive `index_relation.as_ptr()` while the
  guard is in scope.
- Confirm `ec_spire_prepare_coordinator_insert_tuple_payload` preserves the old
  ordering: classify, prepare remote payload, release relation, then perform
  SPI descriptor updates.
- Confirm returned rows remain owned values before tuple conversion.

## Validation

- `bash scripts/unsafe_baseline_report.sh artifacts/unsafe-baseline-before.txt`
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/unsafe_baseline_report.sh`
  - artifact: `artifacts/unsafe-baseline-after.log`
- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/audit-unsafe.log`
- `make fmt-check`
  - artifact: `artifacts/fmt-check.log`
- `git diff --check`
  - artifact: `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18.log`
