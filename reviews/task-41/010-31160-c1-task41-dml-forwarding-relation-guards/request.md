# Review Request: Task 41 DML forwarding relation guards

Code commit: `487e07be194c1963e2f91bac6ca9a3e1f1dd7bf4`

## Summary

This packet continues Task 41 by migrating SPIRE coordinator DML forwarding
entrypoints in `src/lib.rs` from raw `Relation` open/close pairs to
`AccessShareIndexRelation`.

- Migrated batch coordinator insert tuple payload preparation.
- Migrated coordinator update forwarding.
- Migrated coordinator delete preparation.
- Migrated coordinator select forwarding.
- Preserved the old local-node behavior by explicitly dropping the guard before
  local heap helper calls.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: 4590 entries.
- After: 4579 entries.
- Net change: 11 fewer grandfathered unsafe-comment baseline entries.

## Reviewer Focus

- Confirm remote AM helpers only receive `index_relation.as_ptr()` while the
  guard is live.
- Confirm local `node_id == 0` paths still extract `indrelid`, drop the index
  relation, and only then call heap helpers.
- Confirm no SPI write path now runs while the index relation guard is live.

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
