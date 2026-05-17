# 30758 - SPIRE AM Remote Placement Gate

## Summary

This packet reviews commit `6d76971c5dbc227ef9763f72113447ff03bfd7bb`
(`Gate SPIRE AM scan remote placements`).

The slice adds the AM-side counterpart to packet `30757`'s stream delivery
classification. The local manifest loader used by `amrescan` and other
coordinator-local heap consumers now explicitly rejects active placement
directories that contain remote placements with a `remote_row_materialization`
message.

This keeps the legacy local `xs_heaptid` path fail-closed. A mixed coordinator
/ remote placement directory should not drift into `SpireLocalStoreConfig`
validation and fail as if it were only a local-store metadata mismatch; the
blocker is that remote-origin rows need a materialization contract before a
PostgreSQL index scan can return them.

This does not yet implement remote row materialization or route the production
heap-resolved stream through scan opaque state. It makes that boundary explicit
at the current AM entry path.

## Key Files

- `src/am/ec_spire/scan/relation.rs`
- `src/am/ec_spire/scan/tests.rs`
- `src/am/ec_spire/scan/tests/runtime_state.rs`
- `plan/design/spire-production-coordinator-executor.md`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

Packet-local logs are in `artifacts/` and indexed in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo check --no-default-features --features "pg18 pg_test"`
- `git diff --check -- <changed code/docs>`
- Focused `cargo test local_heap_delivery_gate --no-default-features --features pg18`

The focused test target compiles, then direct standalone execution is blocked by
the known pgrx loader issue from packets `30753`, `30756`, and `30757`:
`undefined symbol: SPI_finish`. The raw blocked log is included.

No PostgreSQL server was started for this packet. The changed behavior is a
manifest/placement gate before the existing AM local heap cursor consumes the
active epoch.

## Review Focus

- Is it correct for the current AM local heap path to reject any active remote
  placement until remote row materialization lands?
- Is the gate located at the right boundary, after published snapshot
  validation but before local-store validation?
- Should the error remain a plain string containing `remote_row_materialization`
  or be promoted to a shared constant/status row before AM cursor wiring?
