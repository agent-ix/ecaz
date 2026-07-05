# Review Request: SPIRE Production Remote Index Resolution

Review the narrow production candidate-receive fix in `b2c901d9`:
`Resolve SPIRE production remote index in receive adapter`.

## Change

- Changed `SpireRemoteProductionCandidateReceiveRequest` to carry
  `remote_index_regclass` instead of a coordinator-local `remote_index_oid`.
- Resolve `to_regclass($1)::oid` on the remote connection immediately before
  invoking `ec_spire_remote_search(...)`.
- Added the normalized failure category `remote_index_unavailable` so a missing
  remote index is reported separately from remote query failure.
- Extended the PG18 receive-isolation fixture so one ready remote still returns
  candidates while malformed input, missing conninfo, connect failure, missing
  remote index, remote query failure, decode failure, and batch-validation
  failure stay isolated per node.
- Marked the Phase 11 Stage C checklist item for remote-side regclass
  resolution.

## Why

The production receive adapter cannot use an OID resolved in the coordinator
database. A real remote PostgreSQL node owns its own index OID namespace, so the
request must carry a stable descriptor regclass and resolve it after connecting
to that remote node. This keeps the production state shape aligned with the
remote descriptor catalog and prevents loopback-only OID assumptions from
entering the AM scan path.

## Validation

Raw logs are packet-local under `artifacts/` and summarized in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_spire_prod_receive_isolates_node_failures`
- `git diff --check`

## Review Focus

- Confirm resolving the remote index OID on the remote connection is the right
  production boundary.
- Confirm `remote_index_unavailable` is the right isolated failure category for
  missing remote regclass.
- Confirm the fixture exercises the important distinction between missing
  remote index, remote query failure, candidate decode failure, and candidate
  validation failure without broadening the diagnostic executor.
