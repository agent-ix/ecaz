# Review Request: Task 41 SPIRE index snapshot guards

Code commit: `70f0614ee08008835a261fae4dde999e13e6a1ed`

## Summary

This packet continues Task 41 by migrating the first SPIRE index snapshot
cluster from raw `Relation` open/close ownership to `AccessShareIndexRelation`.

- Migrated object, options, writer identity, boundary replica identity,
  boundary placement, level parameter, scan sanity, health, relation storage,
  and epoch snapshot diagnostics.
- Preserved existing `relation_oid_exists` early-return behavior where present.
- Converted each AM helper call to use `index_relation.as_ptr()` while the guard
  is live, then explicitly dropped the guard before result shaping.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: 4,447 entries.
- After: 4,427 entries.
- Net change: 20 fewer grandfathered unsafe-comment baseline entries.

## Reviewer Focus

- Confirm the relation-existence early returns remained outside guard creation.
- Confirm each migrated snapshot copies owned AM output before the guard is
  dropped and no raw relation pointer escapes.
- Confirm no SPI, environment-variable, or libpq work was moved under a guard.

## Validation

- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/unsafe-comment-audit.log`
- `make fmt-check`
  - artifact: `artifacts/fmt-check.log`
- `git diff --check 70f0614e^ 70f0614e`
  - artifact: `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18.log`
- `bash scripts/unsafe_baseline_report.sh`
  - artifact: `artifacts/baseline-after.log`
