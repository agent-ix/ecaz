# Review Request: Task 41 remote manifest index guards

Code commit: `5d813474c3e09b3917753589eaf55ec497611d13`

## Summary

This packet continues Task 41 by migrating another `src/lib.rs` SPIRE
diagnostic cluster to `AccessShareIndexRelation`.

- Migrated remote-node descriptor readiness, capability, epoch publish, and
  epoch manifest diagnostics to `open_valid_ec_spire_index_guard`.
- Migrated `ec_spire_persist_remote_epoch_manifest`, including error branches
  that previously had manual `index_close` calls.
- Kept the guard alive through the active-epoch recheck inside the SPI write
  transaction.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: 4684 entries.
- After: 4660 entries.
- Net change: 24 fewer grandfathered unsafe-comment baseline entries.

## Reviewer Focus

- Confirm the manifest persistence function keeps the relation live through
  the `spire_active_epoch` recheck and releases it on every return/error path.
- Confirm catalog validation intentionally opens and drops the guard before the
  SPI read because the later query only needs the OID.

## Validation

- `bash scripts/unsafe_baseline_report.sh /private/tmp/tqvector-unsafe-baseline-before-913.txt`
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
