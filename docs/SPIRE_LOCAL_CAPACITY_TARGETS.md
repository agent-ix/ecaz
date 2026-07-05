# SPIRE Local Capacity Targets

This document publishes the Phase 12 local production-readiness capacity
targets for SPIRE. These are local smoke targets for the repository-owned
fixture, not AWS/RDS product-scale capacity claims.

The targets are intentionally conservative. They bound the local readiness
bundle while the typed tuple transport, write-dispatch cancellation,
wide-fanout async write dispatch, and benchmark rows remain separate Phase 12
work.

## Target Profile

Use this profile when a review packet claims local production-readiness smoke
for distributed SPIRE:

| Capacity surface | Local target | Operator setting or control |
| --- | ---: | --- |
| Maximum ready remotes per coordinator query | 8 | `ec_spire.remote_search_max_nodes = 8` |
| Maximum remote leaf PIDs per coordinator query | 256 | `ec_spire.remote_search_max_pids = 256` |
| Maximum selected PIDs per remote node | 64 | `ec_spire.remote_search_max_pids_per_node = 64` |
| Maximum remote tuple payload bytes per row | 1024 | `ec_spire.max_remote_payload_bytes_per_row = 1024` |
| Maximum remote payload rows per batch | 64 | `ec_spire.max_remote_payload_rows_per_batch = 64` |
| Maximum concurrent distributed-read coordinator sessions | 1 | workload admission for the local smoke bundle |
| Maximum concurrent remote-search dispatches across coordinator backends | 8 | `ec_spire.remote_search_max_concurrent_dispatches = 8` |
| Maximum concurrent remote-search dispatches per remote node | 1 | `ec_spire.remote_search_max_concurrent_dispatches_per_node = 1` |
| Maximum concurrent coordinator-routed writer workloads | 1 | workload admission for the local smoke bundle |
| Maximum concurrent work per remote node | 1 read dispatch or 1 prepared write branch | per-node governance plus write workload admission |

The first three fanout targets match the conservative starting point in
`plan/design/spire-libpq-executor-budget.md`. The global dispatch target allows
one fully fanned-out eight-remote query, while the per-node target keeps each
remote to one in-flight read dispatch. The concurrent-read and
concurrent-write targets are v1 local readiness limits. Raise them only in a
packet that includes benchmark or contention logs for the tested machine,
fixture, storage format, and rerank mode.

## Required Local Settings

A packet-local run that uses this profile should set:

```sql
SET ec_spire.remote_search_max_nodes = 8;
SET ec_spire.remote_search_max_pids = 256;
SET ec_spire.remote_search_max_pids_per_node = 64;
SET ec_spire.max_remote_payload_bytes_per_row = 1024;
SET ec_spire.max_remote_payload_rows_per_batch = 64;
SET ec_spire.remote_search_max_concurrent_dispatches = 8;
SET ec_spire.remote_search_max_concurrent_dispatches_per_node = 1;
```

Timeouts are workload-specific and must be recorded in the packet manifest
when set:

```sql
SET ec_spire.remote_search_connect_timeout_ms = <local target>;
SET ec_spire.remote_search_statement_timeout_ms = <local target>;
```

The readiness packet must also record the active consistency mode, node count,
selected PID count, remote fanout, candidate counts, heap rows, timeout/cancel
counts, strict failures, degraded skips, and placement-contention counters
when those harnesses are available.

## Expected Overload Behavior

Remote-search budget and governance overload is fail-closed before conninfo
secret lookup or socket open. Rows blocked by fanout or concurrency caps must
report:

- `status = 'remote_executor_overload'`;
- `next_blocker = 'remote_executor_budget'` for static fanout admission, or
  `next_blocker = 'remote_executor_governance'` for saturated advisory-lock
  governance;
- zero returned candidates for the blocked remote row.

Strict mode treats required overloaded or unavailable remote work as a
distributed-read failure. Degraded mode may skip only the affected remote work
when the query path permits degraded execution, and it must report the skipped
`node_id` and sanitized reason.

The current local readiness target does not rely on unlimited default GUCs.
Although the remote-search budget GUCs default to `0`, meaning uncapped, a
local production-readiness smoke packet should use explicit nonzero caps so an
accidental fanout or cross-backend overload is visible in diagnostics.

Remote tuple-payload caps fail closed with
`remote_payload_too_large` before a decoded typed payload row or batch is
accepted into coordinator merge state. The default row cap is 1024 bytes:
packet `30975` measured 31,510 payload bytes across 200 rows, or roughly 158
bytes per row, and the default rounds the 4x safety margin up to 1 KiB. The
default batch cap is 64 rows, matching the selected-PID local capacity target.
Raise either GUC only with packet-local benchmark evidence for the tuple
projection, fixture, storage format, and rerank mode that need the larger
payload.

## Write Capacity Boundary

Coordinator-routed writes use remote prepared transactions. The v1 local
capacity target admits one coordinator-routed writer workload at a time, with
at most one prepared write branch active per remote node for the local smoke
bundle. Each remote must set `max_prepared_transactions` above zero and leave
free slots for SPIRE plus any other application prepared transactions, as
described in `docs/SPIRE_LIBPQ_RUNBOOK.md`.

Higher writer concurrency is not a Phase 12 local readiness claim until the
placement-table contention fixture, INSERT 2PC cancellation parity, and
wide-fanout async write-dispatch rows have packet-local evidence.

## Claim Boundary

These targets support only the `local production-readiness smoke` evidence
label from `docs/SPIRE_LOCAL_READINESS.md`. They do not claim:

- product-scale capacity;
- managed-service behavior;
- cross-AZ or WAN behavior;
- AWS/RDS latency, throughput, or reliability;
- safe higher read or write concurrency without packet-local measurements.
