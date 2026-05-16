# Review Request: SPIRE CustomScan Executor Stream

Code slice for Step 2 of the ADR-067 CustomScan pivot. This moves
`EcSpireDistributedScan` past the planner-only scaffold: execution now carries a
plan-private contract into `BeginCustomScan`, routes `ExecCustomScan` through
PostgreSQL `ExecScan`, and invokes the existing production
`SpireRemoteFanoutExecutor` result stream.

## Scope

- Adds provider-owned `SpireCustomScanExecState` with serialized index OID,
  constant `real[]` query, LIMIT/top-k, cached production outputs, and rescan
  reset state.
- Extracts the ORDER BY vector query from the transformed sort target and gates
  CustomPath generation on a single relation-var `<op>` constant-`real[]`
  ordering expression.
- Stores index OID/top-k in `custom_private` and the copied query expression in
  `custom_exprs`.
- Makes `ExecCustomScan` use PostgreSQL `ExecScan` so normal scan qual and
  projection handling stay in the executor machinery.
- Calls `remote_search_production_scan_heap_resolution_result_stream(...)` from
  the CustomScan access method.
- Delivers local/materialized heap outputs through the scan tuple slot. Remote
  tuple-payload slot delivery remains fail-closed and is the next implementation
  slice.
- Updates `ec_spire_custom_scan_status()` to report
  `executor_stream_wired_tuple_payload_pending`.
- Updates the Phase 11 task file with packet `30810` progress and the remaining
  ADR-068 tuple-payload slot work.

## Validation

- `cargo test customscan_exec --lib`
  - Proves a remote-placement `SELECT ... ORDER BY ... LIMIT` reaches the
    production executor and fails on the real remote transport gate rather than
    the old scaffold "not wired" error.
- `cargo test custom_scan_status --lib`
- `cargo test customscan_explain --lib`
- `cargo fmt --check`
- `git diff --check HEAD -- src/am/ec_spire/custom_scan.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Review Focus

- Check the CustomScan state allocation/drop pattern: the provider allocates a
  larger state struct whose first field is `CustomScanState`.
- Check the planner/executor contract: `custom_private` carries index OID/top-k;
  `custom_exprs` carries the copied constant query expression.
- Check `ExecScan` integration and guarded access/recheck callbacks. An
  unguarded access callback briefly aborted the backend during development; the
  committed callback is `#[pg_guard]` covered.
- Check the deliberate limitation: parameterized query vectors (`ORDER BY ... $1`)
  and remote-origin tuple-payload slot delivery are still open.

## Artifacts

- `review/30810-spire-customscan-executor-stream/artifacts/manifest.md`
- `review/30810-spire-customscan-executor-stream/artifacts/cargo-test-customscan-exec.log`
- `review/30810-spire-customscan-executor-stream/artifacts/cargo-test-custom-scan-status.log`
- `review/30810-spire-customscan-executor-stream/artifacts/cargo-test-customscan-explain.log`
- `review/30810-spire-customscan-executor-stream/artifacts/cargo-fmt-check.log`
- `review/30810-spire-customscan-executor-stream/artifacts/git-diff-check.log`
