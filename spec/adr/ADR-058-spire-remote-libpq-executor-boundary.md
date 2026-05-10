---
id: ADR-058
title: "SPIRE Remote Libpq Executor Boundary"
status: ACCEPTED
impact: Affects Task 30 Phase 10 remote search execution
date: 2026-05-09
---
# ADR-058: SPIRE Remote Libpq Executor Boundary

## Status

Accepted.

## Context

SPIRE now has SQL-visible remote-search libpq surfaces that can resolve
executor-owned conninfo secrets, open loopback libpq connections, call remote
SPIRE SQL endpoints, decode returned candidate rows, and validate each receive
batch before merge.

That surface is useful for diagnostics and operator proof packets, but it is
not the production AM remote query path. The current implementation uses
blocking `postgres::Client` calls, opens per-query connections, and dispatches
remote nodes serially from SQL functions. It does not enter libpq pipeline
mode, does not multiplex multiple remote nodes concurrently, does not integrate
with PostgreSQL query cancellation as a production executor, and does not own
final SQL result delivery from the AM scan path.

The remote contracts already protect two important invariants:

- raw conninfo is resolved inside executor code from `conninfo_secret_name`
  and is not returned through SQL-visible rows;
- each received candidate batch must pass
  `validate_remote_search_candidate_batch` before entering global merge.

## Decision

The current SQL-visible libpq executor remains a diagnostic/operator executor
for Phase 10. It must not be treated as the production AM remote query path or
used for product performance claims.

A production remote executor remains future work. It needs an accepted design
and implementation that covers concurrent dispatch across ready remote nodes,
libpq pipeline or asynchronous receive, bounded fanout, connect and statement
timeouts, cancellation propagation, cached remote index identity validation,
fail-closed partial-failure behavior, and final row delivery semantics.
The Stage C identity-cache constraints for that production work are captured in
`plan/design/spire-libpq-identity-cache.md`.
The Stage C resource-governance constraints for per-query admission, overload
blocking, and timeout configuration are captured in
`plan/design/spire-libpq-executor-budget.md`.
The production executor state machine, landing sequence, cancellation contract,
and required counters are captured in
`plan/design/spire-production-coordinator-executor.md`.
That contract also owns the PostgreSQL advisory-lock namespace reserved for
first-stage cross-backend dispatch governance:

- global slots reserve class IDs `730000000..=730004095` with object ID `0`;
- per-node slots reserve class IDs `731000000..=731004095` with object ID set
  to the bit-preserving signed representation of `node_id`.

These ranges are part of the production contract until a future async/pipeline
executor replaces advisory-lock governance with an accepted alternative.

## Required Invariants

- SQL-visible diagnostic surfaces may resolve conninfo internally, but must
  never return raw conninfo or raw remote error text.
- Diagnostic receive paths must decode the result contract and validate each
  target-scoped batch with `validate_remote_search_candidate_batch`.
- Any status or review packet must distinguish diagnostic blocking libpq calls
  from production pipeline/concurrent execution.
- Production AM remote scan work must not reuse the diagnostic executor as-is.
- Operator scripts and other extension features must not use the SPIRE
  remote-search governance advisory-lock class ranges.

## Rationale

The diagnostic executor is intentionally narrow: it proves endpoint shape,
secret lookup, remote OID resolution, candidate decoding, and batch validation
without committing the AM scan path to a serial SQL function. Promoting it
directly would bake in the main performance problem Phase 10 is meant to avoid.

Keeping the boundary explicit lets the repo retain useful loopback coverage
while leaving the production executor free to use the right concurrency,
cancellation, timeout, and merge ownership model.

## Consequences

- Phase 10.5's production-bound implementation checklist is deferred because
  this phase chooses diagnostic-only for the existing executor.
- Raw conninfo and receive-batch validation are considered complete for the
  diagnostic executor, not proof that the future production executor is done.
- Future remote performance packets must cite a production executor checkpoint,
  not the SQL-visible diagnostic functions alone.
- Operators can inspect first-stage governance utilization through `pg_locks`
  advisory rows using the reserved class ranges.
