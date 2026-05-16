# Review Request: SPIRE Coordinator Insert Payload Prepare

## Scope

Narrow ADR-069 coordinator-routed INSERT slice after packet `30831`.

This packet wires the coordinator remote-prepare primitive from packet `30830`
to the typed remote tuple-payload INSERT endpoint from packet `30831`. The
coordinator now builds the mutating remote SQL as a call to
`ec_spire_remote_insert_tuple_payload(...)` using the descriptor's
`remote_index_regclass`, a JSON tuple payload, and an explicit column list.

This slice:

- Adds internal `coordinator_insert_prepare_remote_tuple_payload(...)`.
- Resolves the active remote-node descriptor and reuses the existing dispatch
  readiness gate.
- Builds a remote endpoint call rather than table-specific raw INSERT SQL.
- SQL-literal quotes the descriptor remote index regclass, JSON payload, and
  column names before dispatching over libpq.
- Reuses the existing remote transaction, `PREPARE TRANSACTION`, and
  coordinator xact callback resolution path.
- Adds PG18 coverage showing the endpoint call is prepared remotely and the
  local placement row is staged only after remote prepare succeeds.
- Documents the composition in ADR-069 and marks the Phase 11 tracker
  sub-slice.

This intentionally does not install the transparent coordinator DML hook,
construct JSON from real executor tuples, or prove final read-after-insert
through CustomScan. Those remain open Phase 11 work.

## Validation

- `cargo test insert_remote_prepare_tuple_payload --lib`
  - Passed: 1 PG18 test.
- `cargo fmt --check`
  - Passed with the repository's existing stable-rustfmt warnings about
    nightly-only import options.
- `git diff --check`
  - Passed.
- `git diff --cached --check`
  - Passed before the code commit.

## Review Focus

- Confirm the coordinator boundary is correct: descriptor lookup chooses the
  remote SPIRE index, while the remote endpoint derives the target heap.
- Confirm the remote SQL construction is appropriately constrained and quoted
  for descriptor/index, JSON payload, and explicit column list inputs.
- Confirm this preserves the 2PC ordering from ADR-069: remote endpoint call,
  remote prepare, then local placement staging.
- Confirm the test scope is honest: it proves prepared endpoint composition and
  placement staging, but not the final transparent DML hook or CustomScan
  read-after-insert.

## Artifacts

- `review/30832-spire-coordinator-insert-payload-prepare/artifacts/manifest.md`
- `review/30832-spire-coordinator-insert-payload-prepare/artifacts/cargo-test-insert-remote-prepare-tuple-payload-lib.log`
- `review/30832-spire-coordinator-insert-payload-prepare/artifacts/cargo-fmt-check.log`
- `review/30832-spire-coordinator-insert-payload-prepare/artifacts/git-diff-check.log`
- `review/30832-spire-coordinator-insert-payload-prepare/artifacts/git-diff-cached-check.log`
