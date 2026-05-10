# SPIRE Libpq Executor Budget Contract

Status: Phase 11 Stage C design contract

Scope: remote-search libpq dispatch admission, timeout configuration, and
overload diagnostics before production async/pipeline execution.

## Decision

SPIRE remote-search libpq dispatch is admitted through deterministic
executor-budget gates before conninfo secret lookup or socket open.

The first Stage C budget surface is session-scoped:

- `ec_spire.remote_search_max_nodes`
- `ec_spire.remote_search_max_pids`
- `ec_spire.remote_search_max_pids_per_node`
- `ec_spire.remote_search_max_concurrent_dispatches`
- `ec_spire.remote_search_max_concurrent_dispatches_per_node`
- `ec_spire.remote_search_connect_timeout_ms`
- `ec_spire.remote_search_statement_timeout_ms`

The three fanout caps use `0` as unlimited. Nonzero caps are enforced when
building libpq dispatch rows. Rows that are already blocked by descriptor,
epoch, or extension-version gates are not charged against the budget. Ready
rows are admitted in deterministic dispatch order. If admitting a whole row
would exceed a node, total-PID, or per-node-PID cap, the entire row is blocked
with `remote_executor_overload`.

The executor intentionally does not partially truncate a remote row's
`selected_pids`. A libpq row is the unit of remote-node work and carries the
route-budget choice for that node. Partial truncation would create a second,
less visible route budget that could silently change recall. Operators should
reduce upstream fanout or raise the explicit budget instead.

The first cross-query governance surface uses nonblocking PostgreSQL advisory
locks around actual libpq remote work. `remote_search_max_concurrent_dispatches`
limits concurrent remote-search libpq dispatches across coordinator backends.
`remote_search_max_concurrent_dispatches_per_node` limits concurrent dispatches
for one remote node. `0` means unlimited for both settings. Saturated
governance slots report `remote_executor_overload` with
`remote_executor_governance`, before conninfo secret lookup or socket open.

All five budget GUCs default to `0`, so the budget surface is opt-in and
unbounded until an operator sets explicit caps. A conservative production
starting point before capacity-target tuning is `remote_search_max_nodes = 8`,
`remote_search_max_pids = 256`, and `remote_search_max_pids_per_node = 64`,
with the two concurrent-dispatch caps sized from observed remote capacity.

Gate precedence is deterministic: capability/readiness gates run first, budget
admission runs second, and endpoint identity preflight runs only for admitted
rows. This keeps cheap metadata blockers visible before budget pressure while
preserving fail-closed identity checks for rows that can actually dispatch.

## Advisory Lock Namespace

The cross-query governance implementation uses two-argument PostgreSQL advisory
locks through `pg_try_advisory_lock(class_id, object_id)`.

Reserved SPIRE remote-search governance ranges:

- Global dispatch slots use `class_id` values
  `730000000..=730004095` and `object_id = 0`. Slot `n` maps to
  `(730000000 + n, 0)`.
- Per-node dispatch slots use `class_id` values
  `731000000..=731004095`. Slot `n` for `node_id` maps to
  `(731000000 + n, bit_preserving_i32(node_id))`; the `object_id` is the
  signed two's-complement interpretation of the `u32` node identifier.

The range size matches the current
`ec_spire.remote_search_max_concurrent_dispatches*` hard cap of `4096`. Other
extension features, operator scripts, and external runbooks must not use these
class ranges. Operators can inspect current utilization with
`pg_locks WHERE locktype = 'advisory' AND classid::bigint BETWEEN 730000000 AND 731004095`.

## Required Invariants

- Over-budget rows must use `dispatch_action = blocked_before_dispatch`.
- Over-budget rows must report `next_executor_step = remote_executor_budget`.
- Over-budget rows must not resolve `conninfo_secret_name`, build a provider
  lookup key, open a socket, or query endpoint identity.
- Budget diagnostics must report admitted and budget-blocked dispatch/PID
  counts plus the active caps.
- Runtime governance must use nonblocking admission, must release any acquired
  advisory locks when the dispatch returns or errors, and must not hold a global
  slot if a per-node slot cannot be acquired.
- Timeout settings must remain numeric diagnostics and must not expose raw
  conninfo.

## Timeout Contract

`remote_search_connect_timeout_ms` is applied to the parsed postgres connection
configuration when nonzero. `remote_search_statement_timeout_ms` is applied
after connection open with a bounded numeric `SET statement_timeout = ...`
statement when nonzero.

The current diagnostic executor still uses blocking `postgres::Client` calls.
This budget contract does not claim async execution by itself; it removes the
unbounded fanout, cross-backend overload, and timeout gaps that would otherwise
be baked into the next async/pipeline executor slice.

## Remaining Work

- Propagate PostgreSQL cancellation into in-flight remote work.
- Replace serial diagnostic dispatch with production async or libpq pipeline
  execution.
