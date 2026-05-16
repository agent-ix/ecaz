# Review Request: SPIRE Coordinator Insert Read-After-CustomScan

## Scope

Narrow ADR-069 coordinator-routed INSERT validation slice after packet `30833`.

This packet adds the first PG18 multicluster read-after-insert proof for the
helper-based coordinator INSERT path. It also resolves the packet `30833`
operator-facing status feedback by making the SQL helper report that the remote
transaction is prepared and awaiting local commit, rather than implying the
remote row is already durable.

This slice:

- Adds `scripts/run_spire_multicluster_insert_read_after_customscan_pg18.sh`.
- Starts separate coordinator and remote PG18 clusters.
- Creates matching coordinator and remote SPIRE indexes, rewrites coordinator
  leaf placement to remote node `2`, and registers the remote descriptor.
- Invokes `ec_spire_prepare_coordinator_insert_tuple_payload(...)`.
- Verifies the remote tuple-payload endpoint committed row `303` after the
  coordinator transaction resolved the prepared remote transaction.
- Verifies the coordinator placement-directory row was staged at the
  coordinator search epoch.
- Refreshes the remote descriptor identity after the remote index advances.
- Confirms `SELECT ... ORDER BY embedding <#> ARRAY[...] LIMIT 1` plans through
  `Custom Scan (EcSpireDistributedScan)` and returns the inserted remote row.
- Advances the post-staging SQL helper status to
  `remote_insert_prepared_pending_local_commit` with next step
  `await_local_commit`.
- Updates ADR-069 and the Phase 11 tracker with the read-after-insert coverage.

This intentionally still does not install the transparent `INSERT INTO table`
hook. The hook remains responsible for constructing canonical primary-key bytes,
ADR-063 source identity, tuple JSON, and requested column lists before invoking
the helper.

## Validation

- `scripts/run_spire_multicluster_insert_read_after_customscan_pg18.sh --skip-install --artifact-dir review/30834-spire-coordinator-insert-read-after-customscan/artifacts --run-id 20260511T172633Z-rerun4`
  - Passed.
  - Key result: `Custom Scan (EcSpireDistributedScan)` returned
    `303,remote inserted via coordinator`.
- `cargo test prepare_coordinator_insert_tuple_payload --lib`
  - Passed: 1 PG18 test.
- `cargo fmt --check`
  - Passed with the repository's existing stable-rustfmt warnings about
    nightly-only import options.
- `git diff --check`
  - Passed.

## Review Focus

- Confirm the multicluster script is a useful read-after-insert proof for the
  helper path before the transparent DML hook lands.
- Confirm the epoch alignment in the fixture is legitimate: coordinator stages
  the placement at epoch `2`, the remote INSERT advances the remote index to
  epoch `2`, then the descriptor identity is refreshed before CustomScan.
- Confirm the post-staging status names distinguish prepared/pending-local-commit
  from committed/durable remote state.
- Confirm the Phase 11 tracker still leaves transparent coordinator INSERT hook
  work open.

## Artifacts

- `review/30834-spire-coordinator-insert-read-after-customscan/artifacts/manifest.md`
- `review/30834-spire-coordinator-insert-read-after-customscan/artifacts/multicluster-insert-read-after-customscan.log`
- `review/30834-spire-coordinator-insert-read-after-customscan/artifacts/remote-postgres.log`
- `review/30834-spire-coordinator-insert-read-after-customscan/artifacts/coord-postgres.log`
- `review/30834-spire-coordinator-insert-read-after-customscan/artifacts/cargo-test-prepare-coordinator-insert-tuple-payload-lib.log`
- `review/30834-spire-coordinator-insert-read-after-customscan/artifacts/cargo-fmt-check.log`
- `review/30834-spire-coordinator-insert-read-after-customscan/artifacts/git-diff-check.log`
