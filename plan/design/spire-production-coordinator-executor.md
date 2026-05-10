# SPIRE Production Coordinator Executor Plan

Status: Phase 11 Stage C design checkpoint
Date: 2026-05-10
Scope: production remote-search fanout from the SPIRE coordinator AM path

## Goal

Replace the diagnostic SQL-visible libpq executor shape with a production
coordinator executor that can send remote SPIRE search work to multiple ready
PostgreSQL nodes, overlap remote work, observe local cancellation, enforce
resource limits, and return validated candidate batches to the existing merge
and remote-heap-resolution contracts.

This document is intentionally broad. It is the quality gate for the next code
slices, not proof that the production executor is done.

## Boundary

The existing `ec_spire_remote_search_libpq_*` functions remain diagnostic
surfaces. They are useful for packet evidence because they expose request,
budget, secret, connection, identity, receive, and summary rows. They do not
become the production executor.

The production executor owns a per-query state object inside the coordinator
scan path. Diagnostic SQL may later read summaries derived from that state, but
production execution must not depend on running SQL-visible helper functions in
the hot path.

Raw conninfo stays hidden behind `conninfo_secret_name` resolution. Any
diagnostic status can include sanitized provider names, node IDs, blocker
strings, counts, and libpq SQLSTATE-style categories, but not raw conninfo or
raw remote error text.

## Executor State Model

The first production shape should be explicit and small:

```text
SpireRemoteFanoutExecutor
  request: query vector, requested epoch, top-k, consistency mode
  dispatches: Vec<SpireRemoteDispatch>
  identity_cache: bounded executor-local cache
  limits: per-query caps, cross-query governance caps, timeouts
  cancellation: local cancel token + remote cancel handles
  counters: fanout, candidates, heap rows, bytes, timeouts, cancellations

SpireRemoteDispatch
  node_id
  selected_pids
  descriptor_generation
  remote_index_regclass
  remote_index_identity
  conninfo_secret_name
  served_epoch
  state
```

Dispatch state is monotonic:

```text
planned
blocked_before_dispatch
secret_resolved
connected
identity_validated
sent
receiving
received
validated
heap_resolution_pending
completed
failed_strict
skipped_degraded
cancelled
```

Every state transition must preserve the packet-friendly blocker vocabulary
already used by the diagnostic surfaces: descriptor, epoch window, extension
version, executor budget, executor governance, conninfo secret, connection,
remote endpoint identity, remote endpoint result, remote heap resolution, and
coordinator merge.

## Landing Sequence

### C0: State Contract

Define the production state structs and conversion from existing dispatch-plan
rows without opening sockets. The state must expose a summary row equivalent to
the diagnostic pipeline summary, but backed by production state.

Verification:

- Rust unit tests for state transition legality.
- PG18 dry-path test proving no conninfo lookup or socket open occurs before
  dispatch admission.

### C1: Async / Pipeline Transport Adapter

Implement a transport adapter that can overlap at least two ready remote
dispatches. The preferred v1 shape is libpq async send/receive or pipeline mode
through a small adapter boundary rather than direct blocking `postgres::Client`
calls. If a temporary threaded adapter is used for local validation, it must be
documented as a bridge and blocked from production-readiness claims.

Verification:

- Local two-remote fixture where one remote is instrumented slow and the other
  returns first.
- Packet log showing ready remote B is not serialized behind slow remote A.
- Diagnostic counters include send timestamp, first-row timestamp, complete
  timestamp, and timeout/cancel category per node.

### C2: Cancellation And Timeouts

Propagate PostgreSQL query cancellation to outstanding remote work. On local
cancel or local statement timeout, the executor must stop accepting new remote
work, request cancellation for in-flight remote queries when libpq provides a
cancel handle, drain or close remote connections safely, release governance
locks, and report sanitized cancellation counters.

Remote statement timeout remains remote-owned and should surface as a remote
timeout category, distinct from local cancel and local statement timeout.

Verification:

- PG18/local fixture proving a local cancel releases global and per-node
  governance slots.
- Fixture proving remote statement timeout skips or fails the node according to
  strict/degraded mode without masking the timeout as identity or connection
  failure.

### C3: Production Identity Cache Use

Move the bounded executor-local endpoint identity cache from diagnostic proof
into the production state. Reuse is allowed only under the key and invalidation
rules in `plan/design/spire-libpq-identity-cache.md`.

Verification:

- Existing diagnostic cache matrix remains green.
- Production-state test proves compact candidate and remote heap receive share
  one identity decision.
- Fingerprint mismatch invalidates the cache and never reseats descriptor
  identity from the remote.

### C4: Strict / Degraded Failure Semantics

Normalize all remote failures into explicit strict and degraded outcomes:

- strict mode: fail closed before merge when a required remote node cannot
  prove descriptor, epoch, version, identity, endpoint, or heap correctness;
- degraded mode: skip only the failing node or dispatch, preserve exact reason,
  and keep local and other ready remote candidates eligible.

Verification:

- Fault matrix for auth or certificate failure, connection reset, remote
  backend termination, remote timeout, local cancel, network partition,
  endpoint version skew, stale epoch, fingerprint mismatch, and missing remote
  index.

### C5: AM Scan Integration

Wire production remote fanout into the coordinator scan path behind an explicit
readiness gate. The scan path must merge local and remote compact candidates
only after candidate batch validation and must defer final SQL row readiness
until Stage D remote heap resolution is production-correct.

Verification:

- One coordinator plus two remote PostgreSQL nodes can return one ordered
  candidate stream.
- If Stage D is not yet complete, final row delivery reports the existing
  `requires_remote_heap_resolution` blocker rather than pretending remote rows
  are SQL-ready.

### C6: Operator And Harness Readiness

Expose production-state summaries for operators and packet capture without
opening extra remote sockets. The dry pipeline entrypoint should stay dry; live
diagnostic probes should remain opt-in.

Verification:

- `ecaz` local multi-instance command captures recall, latency p50/p95/p99,
  fanout, candidate, heap, timeout, cancel, strict failure, degraded skip, and
  byte counters into packet-local logs.

## Performance Risks To Avoid

- Serial remote execution hidden behind helper functions.
- Per-query repeated endpoint identity round trips when the descriptor,
  generation, identity, epoch, and endpoint contract have already been
  validated inside the same executor state.
- Opening sockets or resolving secrets for rows already blocked by capability,
  budget, or governance gates.
- Partial PID truncation inside a dispatch row, which creates an implicit recall
  budget separate from the visible route budget.
- Holding governance slots while waiting on unrelated nodes or while blocked on
  local cancellation cleanup.
- Treating diagnostic live probes as performance evidence for the production
  AM path.

## Required Production Counters

The production executor must count at least:

- planned, admitted, budget-blocked, governance-blocked, sent, received,
  validated, failed-strict, skipped-degraded, cancelled dispatches;
- selected PIDs by node and `(node_id, local_store_id)`;
- candidate rows returned and candidate rows accepted after validation;
- remote heap rows requested, found, missing, dead, stale, and failed;
- endpoint identity cache hits, misses, invalidations, and live mismatches;
- connection opens, reuses, failures, connect timeouts, remote statement
  timeouts, local cancels, and local statement timeouts;
- remote object bytes and row-locator bytes where available.

## Open Decisions

- Exact transport implementation: direct libpq async/pipeline FFI, a narrow
  crate wrapper, or a temporary local validation bridge. The production gate
  should only accept direct overlapped libpq semantics or an explicitly accepted
  equivalent.
- Connection reuse policy after C1: per-query connect/close is acceptable for
  correctness, but performance readiness needs either bounded reuse or measured
  evidence that connect cost is outside the query hot path.
- Shared identity cache: deferred until a memory cap, lock-order contract, and
  descriptor-write invalidation path are accepted.
