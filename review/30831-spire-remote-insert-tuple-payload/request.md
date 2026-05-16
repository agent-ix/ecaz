# Review Request: SPIRE Remote Insert Tuple Payload

## Scope

Narrow ADR-069 coordinator-routed INSERT slice after packet `30830`.

This packet adds the remote-side typed INSERT endpoint that the coordinator can
call inside the prepared remote transaction. The goal is to replace
table-specific raw INSERT SQL with a stable remote SPIRE endpoint before the
transparent coordinator DML hook is wired.

This slice:

- Adds `ec_spire_remote_insert_tuple_payload(index_oid, row_payload,
  requested_columns)`.
- Derives the target heap relation from the supplied remote SPIRE index.
- Validates the explicit column list against ordinary heap attributes and
  rejects empty column lists.
- Quotes requested column identifiers before building dynamic SQL.
- Projects the JSON payload through `jsonb_populate_record`, so PostgreSQL
  type input owns scalar conversion, including `ecvector`.
- Inserts exactly the requested columns and returns `ready`, inserted count,
  heap relation OID, and payload column count.
- Documents the endpoint in ADR-069 and marks the Phase 11 tracker sub-slice.

This intentionally does not install the coordinator-side DML hook, construct
JSON from executor tuples, call this endpoint through the remote-prepare
primitive, or prove read-after-insert through CustomScan. Those remain open
Phase 11 work.

## Validation

- `cargo test remote_insert_tuple_payload --lib`
  - Passed: 1 PG18 test.
- `cargo fmt --check`
  - Passed with the repository's existing stable-rustfmt warnings about
    nightly-only import options.
- `git diff --check`
  - Passed.
- `git diff --cached --check`
  - Passed before the code commit.

## Review Focus

- Confirm the endpoint is the right remote-side boundary for coordinator
  INSERT before wiring the coordinator DML hook.
- Confirm deriving the heap relation from the remote SPIRE index is appropriate
  and avoids adding a separate remote table-name contract.
- Confirm JSON projection through `jsonb_populate_record` is acceptable for the
  v1 scalar tuple payload contract.
- Confirm dynamic SQL is constrained by catalog-validated column names and
  identifier quoting.
- Confirm the tracker still leaves tuple-to-JSON construction, coordinator
  hook wiring, remote prepare invocation, placement staging, and CustomScan
  read-after-insert open.

## Artifacts

- `review/30831-spire-remote-insert-tuple-payload/artifacts/manifest.md`
- `review/30831-spire-remote-insert-tuple-payload/artifacts/cargo-test-remote-insert-tuple-payload-lib.log`
- `review/30831-spire-remote-insert-tuple-payload/artifacts/cargo-fmt-check.log`
- `review/30831-spire-remote-insert-tuple-payload/artifacts/git-diff-check.log`
- `review/30831-spire-remote-insert-tuple-payload/artifacts/git-diff-cached-check.log`
