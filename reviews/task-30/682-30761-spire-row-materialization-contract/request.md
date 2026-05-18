# 30761 - SPIRE Row Materialization Contract

## Summary

This packet reviews commit `bdc6349797a04090a0486fa664529fb2a9e4f23d`
(`Expose SPIRE remote row materialization contract`).

The slice makes the remote-origin row delivery boundary SQL-visible through
`ec_spire_remote_search_row_materialization_contract()`. The contract records
that an origin-node heap coordinate is never a legal coordinator `xs_heaptid`.
For the current PostgreSQL index AM path, remote-origin delivery must first
materialize a shadow/proxy row whose TID belongs to the same coordinator heap
relation being scanned. FDW/custom executor tuple delivery is called out as a
future non-AM integration, not a mixed mode inside the current AM cursor path.

This does not implement remote row materialization yet. It pins the contract
that the implementation must satisfy before remote-origin rows can be returned
from a PostgreSQL index scan.

## Key Files

- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/tests.rs`
- `src/lib.rs`
- `plan/design/spire-production-coordinator-executor.md`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

Packet-local logs are in `artifacts/` and indexed in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `git diff --check -- <changed code/docs>`
- `cargo check --no-default-features --features pg18`
- `cargo check --no-default-features --features "pg18 pg_test"`
- `cargo test row_materialization_contract --no-default-features --features pg18`
  compiled, then hit the known direct-test pgrx loader failure:
  `undefined symbol: SPI_finish`.

No PostgreSQL server or distributed fixture was started for this packet. The
change is a contract/diagnostic surface that constrains the next implementation
slice.

## Review Focus

- Is the same-indexed-heap shadow/proxy row requirement the right
  AM-compatible contract for remote-origin result delivery?
- Is the contract explicit enough that origin-node heap coordinates cannot be
  accidentally treated as coordinator `xs_heaptid` values?
- Is FDW/custom executor delivery kept sufficiently separate from the v1 index
  AM path?
- Are the blocker/status names consistent with packets `30757`, `30758`, and
  `30760`?
