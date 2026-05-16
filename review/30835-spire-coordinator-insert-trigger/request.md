# Review Request: SPIRE Coordinator Insert Trigger

## Scope

Narrow ADR-069 coordinator-routed INSERT slice after packet `30834`.

This packet adds the first transparent `INSERT INTO coordinator_table ...`
front door using the trigger-based approach allowed by ADR-069. The trigger is
opt-in and intentionally scoped to the v1 table shape: bigint primary key,
`ecvector` embedding, and 16-byte bytea source identity.

This slice:

- Adds `ec_spire_coordinator_insert_forward_trigger()`.
- Adds `ec_spire_enable_coordinator_insert(table_oid, index_oid, pk_column,
  embedding_column, source_identity_column default 'source_identity')`.
- Validates that the supplied index belongs to the table and uses the
  `ec_spire` access method.
- Validates v1 column types: bigint primary key, `ecvector` embedding, and
  bytea source identity.
- Installs a `BEFORE INSERT FOR EACH ROW` trigger.
- Encodes the bigint primary key via PostgreSQL `int8send`.
- Casts the `ecvector` embedding to `real[]` for classifier input.
- Builds tuple JSON with `to_jsonb(NEW)` and the requested column list from
  live heap attributes.
- Calls `ec_spire_prepare_coordinator_insert_tuple_payload(...)`, reusing the
  reviewed remote tuple-payload prepare + 2PC path.
- Returns `NULL` from the trigger so remote-owned rows are not mirrored in the
  coordinator heap.
- Adds focused PG18 coverage for `INSERT INTO` interception, prepared remote
  transaction, placement staging, and coordinator heap suppression.
- Documents the remaining gap: automatic post-commit descriptor epoch/identity
  refresh is still required before read-after-INSERT is fully transparent.

This intentionally does not yet add the multicluster `INSERT INTO` +
CustomScan read-after-insert fixture. That should land after descriptor refresh
is automatic instead of manually re-registering the descriptor as packet `30834`
does.

## Validation

- `cargo test enable_coordinator_insert_trigger --lib`
  - Passed: 1 PG18 test.
- `cargo fmt --check`
  - Passed with the repository's existing stable-rustfmt warnings about
    nightly-only import options.
- `git diff --check`
  - Passed.
- `git diff --cached --check`
  - Passed before the code commit.

## Review Focus

- Confirm the trigger-based front door is an acceptable narrow implementation
  of ADR-069's trigger option before a lower-level ModifyTable hook exists.
- Confirm the v1 type constraints are explicit enough: bigint PK, `ecvector`
  embedding, bytea source identity.
- Confirm returning `NULL` from the `BEFORE INSERT` trigger preserves the
  "no coordinator-side mirror row" invariant.
- Confirm the tracker correctly keeps automatic descriptor refresh and
  multicluster `INSERT INTO` read-after-insert coverage open.

## Artifacts

- `review/30835-spire-coordinator-insert-trigger/artifacts/manifest.md`
- `review/30835-spire-coordinator-insert-trigger/artifacts/cargo-test-enable-coordinator-insert-trigger-lib.log`
- `review/30835-spire-coordinator-insert-trigger/artifacts/cargo-fmt-check.log`
- `review/30835-spire-coordinator-insert-trigger/artifacts/git-diff-check.log`
