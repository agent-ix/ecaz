# 30757 - SPIRE Production AM Delivery Contract

## Summary

This packet reviews commit `ff3bf705882102ced95ee42e5ac45f1e480ae8fa`
(`Classify SPIRE production AM delivery outputs`).

The slice addresses the Stage D tuple-delivery boundary before wiring the
production scan stream into `amrescan` / `amgettuple`. It keeps the packet
`30756` single ordered result stream, but adds an AM-delivery summary that
classifies outputs into:

- local coordinator heap TIDs that are safe to return through `xs_heaptid`;
- remote-origin rows that must block on `remote_row_materialization`; and
- upstream-blocked streams whose existing failure/blocker should be preserved.

This is intentionally fail-closed. If any remote-origin output remains in the
top-k stream, AM tuple delivery reports
`requires_remote_row_materialization` instead of returning a partial local
prefix. That preserves global ordering and prevents PostgreSQL from treating an
origin-node heap coordinate as a coordinator-local heap TID.

This does not yet implement remote row materialization or cursor the stream
from AM scan opaque state. Those remain the next Stage D production-readiness
items.

## Key Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/root/tests.rs`
- `plan/design/spire-production-coordinator-executor.md`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

Packet-local logs are in `artifacts/` and indexed in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo check --no-default-features --features "pg18 pg_test"`
- `git diff --check -- <changed code/docs>`
- Focused `cargo test production_scan_am_delivery --no-default-features --features pg18`

The focused test target compiles, then direct standalone execution is blocked by
the known pgrx loader issue from packets `30753` and `30756`:
`undefined symbol: SPI_finish`. The raw blocked log is included.

No PostgreSQL server was started for this packet. The change is a Rust-side
classification contract on the stream already SQL-wrapped and PG18-verified in
packet `30756`.

## Review Focus

- Is the fail-closed rule correct when a merged top-k stream contains any
  remote-origin output?
- Is `node_id == SPIRE_LOCAL_NODE_ID` plus `coordinator_local_heap` a sufficient
  predicate for outputs that may become `xs_heaptid`?
- Is `remote_row_materialization` the right next blocker name and boundary for
  remote-origin output delivery?
