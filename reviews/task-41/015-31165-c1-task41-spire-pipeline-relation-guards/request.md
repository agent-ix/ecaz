# Review Request: Task 41 SPIRE pipeline relation guards

Code commit: `ed39f5d39f13fbb1f4e0956d606645e7034e455a`

## Summary

This packet continues Task 41 by migrating the SPIRE remote pipeline, receive,
merge-input, and heap-resolution diagnostic cluster in `src/lib.rs` from raw
`Relation` open/close pairs to `AccessShareIndexRelation`.

- Replaced the remote pipeline helper's long raw relation lifetime with short
  guard scopes around AM helper calls.
- Kept environment-variable lookups and optional libpq connection probes outside
  any live relation guard.
- Migrated receive-plan, receive-summary, merge-input, local heap-resolution,
  heap-resolution summary, and local heap-candidate diagnostics to
  `open_valid_ec_spire_index_guard`.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: 4,493 entries.
- After: 4,479 entries.
- Net change: 14 fewer grandfathered unsafe-comment baseline entries.

## Reviewer Focus

- Confirm the remote pipeline helper no longer keeps an index relation open
  while probing environment variables or opening libpq connections.
- Confirm every AM helper receives `index_relation.as_ptr()` only while the
  guard is live.
- Confirm the new multiple-open pattern in the live pipeline path is acceptable
  for diagnostics: dispatch/connection plan, optional identity cache, and
  optional coordinator result each use their own short guard.

## Validation

- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/unsafe-comment-audit.log`
- `make fmt-check`
  - artifact: `artifacts/fmt-check.log`
- `git diff --check ed39f5d3^ ed39f5d3`
  - artifact: `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18.log`
- `bash scripts/unsafe_baseline_report.sh`
  - artifact: `artifacts/baseline-after.log`
