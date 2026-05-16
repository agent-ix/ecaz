# Review Request: SPIRE Coordinator Insert Remote Prepare

## Scope

Narrow ADR-069 coordinator-routed INSERT slice after packet `30829`.

This packet adds the first mutating internal 2PC primitive for coordinator
INSERT. It consumes the dispatch-readiness plan from packet `30829`, opens the
Stage C libpq transport to the chosen remote, sends caller-provided remote
INSERT SQL inside a remote transaction, issues `PREPARE TRANSACTION`, and
registers coordinator transaction callbacks to resolve the prepared remote
transaction as `COMMIT PREPARED` or `ROLLBACK PREPARED`.

This slice:

- Adds `coordinator_insert_prepare_remote_sql(...)` as an internal Rust
  primitive, re-exported through `am`.
- Reuses the existing remote descriptor, conninfo-secret, libpq timeout, and
  advisory-governance paths.
- Generates a coordinator-scoped prepared transaction GID from index OID,
  node id, served epoch, top transaction id, and backend pid.
- Returns `remote_insert_prepared` with next step
  `local_placement_directory_write` only after remote prepare succeeds.
- Adds a PG18 test-only helper that stages an `ec_spire_placement` row after
  remote prepare succeeds.
- Enables `max_prepared_transactions = 10` for pg_test so PG18 prepared-xact
  coverage can run.
- Documents the primitive in ADR-069 and marks the Phase 11 tracker sub-slice.

This intentionally does not expose raw remote SQL as a production SQL surface,
build generic tuple-to-remote-INSERT statements, install the transparent
coordinator DML hook, or prove read-after-insert through CustomScan. Those
remain open Phase 11 work.

## Validation

- `cargo test insert_remote_prepare --lib`
  - Passed: 1 PG18 test.
- `cargo fmt --check`
  - Passed with the repository's existing stable-rustfmt warnings about
    nightly-only import options.
- `git diff --check`
  - Passed.
- `git diff --cached --check`
  - Passed before the code commit.

## Review Focus

- Confirm the 2PC boundary is correct: remote INSERT is prepared before local
  placement-directory staging, and prepared transaction resolution follows the
  coordinator transaction outcome.
- Confirm callback failure behavior is acceptable for this slice: callback
  resolution failures are swallowed in the xact callback and leave the prepared
  transaction for normal PostgreSQL prepared-transaction recovery.
- Confirm the helper remains internal/test-only and does not create a
  production raw-SQL injection surface.
- Confirm the `max_prepared_transactions` pg_test config change is scoped
  appropriately for prepared-xact coverage.
- Confirm the Phase 11 tracker still leaves transparent DML hook,
  tuple-to-remote-INSERT construction, and CustomScan read-after-insert open.

## Artifacts

- `review/30830-spire-coordinator-insert-remote-prepare/artifacts/manifest.md`
- `review/30830-spire-coordinator-insert-remote-prepare/artifacts/cargo-test-insert-remote-prepare-lib.log`
- `review/30830-spire-coordinator-insert-remote-prepare/artifacts/cargo-fmt-check.log`
- `review/30830-spire-coordinator-insert-remote-prepare/artifacts/git-diff-check.log`
- `review/30830-spire-coordinator-insert-remote-prepare/artifacts/git-diff-cached-check.log`
