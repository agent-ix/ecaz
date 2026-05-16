# Review Request: Task 41 more SQL diagnostic index guards

Code commit: `a8c2eab3ccd337f1a6625ccc48034131d8961c74`

## Summary

This packet continues the Task 41 `src/lib.rs` SQL diagnostic guard migration.

- Migrated a cluster of SPIRE and IVF diagnostic SQL functions to
  `open_valid_ec_*_index_guard`.
- Removed manual `index_close` calls from those functions, including the
  immediate validation-only close in
  `ec_spire_register_remote_node_descriptor`.
- Kept the existing raw helper compatibility layer for remaining callers.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: 4700 entries.
- After: 4684 entries.
- Net change: 16 fewer grandfathered unsafe-comment baseline entries.

## Reviewer Focus

- Confirm the migrated functions do not extend relation lifetime past the
  intended diagnostic read, especially the descriptor registration validator
  block.
- Confirm the remaining raw helper callers are unchanged and still own their
  manual close responsibility until later slices migrate them.

## Validation

- `bash scripts/unsafe_baseline_report.sh /private/tmp/tqvector-unsafe-baseline-before-912.txt`
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/unsafe_baseline_report.sh`
  - artifact: `artifacts/unsafe-baseline-after.log`
- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/audit-unsafe.log`
- `make fmt-check`
  - artifact: `artifacts/fmt-check.log`
- `git diff --check HEAD^ HEAD`
  - artifact: `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18.log`
