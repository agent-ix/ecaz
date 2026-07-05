# 30756 - SPIRE Production Scan Result Stream

## Summary

This packet reviews commit `dc724acf6dedab0f982306c7e7cc13024634da16`
(`Add SPIRE production scan result stream`).

The slice addresses the 30754 reviewer P3 about avoiding SQL summary rows as
the internal AM contract. It adds a Rust-side
`SpireRemoteProductionScanResultStream` with:

- the existing heap-resolution summary row; and
- ordered heap-resolved output rows carrying requested/served epoch, node ID,
  heap block/offset, score, heap lookup owner, vec-id bytes, and opaque row
  locator bytes.

`ec_spire_remote_search_production_scan_heap_resolution_summary(...)` keeps its
external shape, but now serializes from this stream. The final Stage D cursor
slice can consume the stream directly from `amrescan` / `amgettuple` instead of
round-tripping through SQL.

This does not yet move tuple delivery into the AM callbacks. The next blocking
item remains wiring the stream into scan opaque state and deciding the exact
local-vs-remote row delivery contract.

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
- `cargo pgrx install --test -c /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features "pg18 pg_test"`
- PG18 focused SQL wrapper:
  - reset `ecaz_30756`
  - `CREATE EXTENSION ecaz CASCADE`
  - `SELECT tests.test_ec_spire_prod_scan_heap_resolution()`

The focused SQL wrapper verifies the existing production heap-resolution summary
surface still runs after the stream refactor.

The new pure Rust unit test
`production_scan_result_outputs_preserve_heap_resolution_origin` compiles, but
direct `cargo test ... --features pg18` still cannot launch the pgrx-linked test
binary because of the known packet `30753` loader issue:
`undefined symbol: SPI_finish`. The raw blocked log is included.

I also attempted the preferred operator surface
`target/release/ecaz dev install ecaz-pg-test --pg 18`; the existing
`target/release/ecaz` binary failed before installation while resolving the repo
root from `crates/ecaz-cli`. I fell back to the known working `cargo pgrx
install` path for this packet. That operator CLI issue is separate from this
result-stream refactor and should be fixed before the final production harness
gate.

## Review Focus

- Is `SpireRemoteProductionScanResultStream` narrow enough to be the AM-facing
  Stage D contract?
- Does the output row preserve enough origin-node heap information for the next
  `amrescan` / `amgettuple` cursor slice without exposing remote locator
  interpretation to the coordinator?
- Should the next slice split local heap TID outputs from remote-origin outputs
  before attempting final tuple delivery?
