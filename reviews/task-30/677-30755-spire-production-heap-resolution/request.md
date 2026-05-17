# 30755 - SPIRE Production Heap Resolution Proof

## Summary

This packet reviews commit `cc8fc351f7bdd3bc3e5fb5ddd74d46b0b4eb3848`
(`Add SPIRE production heap resolution proof`).

The slice adds the first Stage D production heap-resolution proof surface:
`ec_spire_remote_search_production_scan_heap_resolution_summary(...)`. It
starts from the same scan-router selected leaf PIDs as `amrescan`, runs the
production compact-candidate receive path, gates remote heap receive on
`CandidateReceiveReady`, asks the origin node to resolve heap visibility under
its PostgreSQL snapshot, exact-reranks visible heap rows, and merges ready local
plus remote heap-resolved candidates with the existing deterministic ordering.

Remote row locators remain opaque to the coordinator. The origin SQL surface
interprets the locator and returns heap block/offset diagnostics only after
visibility resolution. In strict mode, an indexed remote locator that no longer
resolves to a visible heap row fails as `remote_heap_resolution_failed` with
`remote_heap_resolution` as the next blocker.

This is still a summary/proof surface. The final Stage D integration remains:
move the Rust heap-resolved result stream into `amrescan` / `amgettuple` without
making the SQL summary row the internal AM contract. The Phase 11 task file now
tracks that narrower Rust-side handoff/result-stream state explicitly.

## Key Files

- `src/am/ec_spire/root/hierarchy_snapshots.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/root/types.rs`
- `src/am/mod.rs`
- `src/lib.rs`
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
- Focused PG18 SQL wrapper:
  - reset packet database
  - start PG18
  - `CREATE EXTENSION ecaz CASCADE`
  - `SELECT tests.test_ec_spire_prod_scan_heap_resolution()`

The focused wrapper creates a loopback remote index and coordinator index with
matching deterministic `rabitq` shape, rewrites one coordinator leaf placement
to node 2, registers the remote descriptor, and verifies the visible-row path:

```text
effective_nprobe = 2
selected_pid_count = 2
local_pid_count = 1
remote_pid_count = 1
dispatch_count = 1
compact_candidate_count = 1
remote_heap_ready_dispatch_count = 1
remote_heap_candidate_count = 1
local_heap_candidate_count = 1
returned_candidate_count = 2
result_source = remote_heap_candidates
final_heap_fetch_status = remote_ready
status = ready
```

The same wrapper then deletes the remote heap rows after index build and
verifies the missing-row failure path:

```text
status = remote_heap_resolution_failed
next_blocker = remote_heap_resolution
remote_heap_failed_dispatch_count = 1
```

Broad `cargo pgrx test` was not run for this packet. The focused SQL-wrapper
path was used because the recent direct `cargo pgrx test` path is still blocked
by the existing standalone test-binary loader issue tracked in packet `30753`
(`undefined symbol: SPI_finish`). The broader PG18 pgrx pass across coordinator
fanout call sites is now tracked as follow-up once that loader issue is fixed.

## Review Focus

- Does the origin-node heap visibility boundary keep remote locators opaque
  enough for final AM tuple delivery?
- Is the production executor state transition from `CandidateReceiveReady` to
  `RemoteHeapReady` / `RemoteHeapFailed` correct under strict and degraded
  consistency?
- Is the summary/proof surface acceptable as a Stage D checkpoint while the
  final `amrescan` / `amgettuple` result-stream state is still open?
