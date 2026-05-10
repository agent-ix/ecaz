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

The C1 implementation lands each executor stage with the same summary shape:
the previous ready state is the next stage's pending count, applying a stage
result is a single monotonic transition, and summaries expose
`pending` / `sent` / `ready` / `failed` counters plus a first failure category.
`TransportReady` / `TransportFailed` and
`CandidateReceiveReady` / `CandidateReceiveFailed` are the reference pattern
for future cancellation, cache-reuse, strict/degraded, and heap-resolution
states. New stages should add counters rather than overloading a status string
when operators need to distinguish partial progress from terminal failure.
Local-cancel transitions must clear any retained candidate batch and candidate
count on the dispatch, matching candidate-receive failure behavior, so a
previously ready but cancelled dispatch cannot contribute to compact merge or
Stage D heap resolution.

As of 2026-05-10, C0/C1 is materially composable through packets 30724-30736:
transport, per-node isolation, executor-owned compact receive, remote-side
regclass resolution, strict merge preconditions, cancellation batch cleanup,
and routing-only selected-leaf PID handoff have landed. The production gate
remains open until C2 cancellation propagation, C3 production identity-cache
reuse, C4 strict/degraded AM-boundary semantics, C5 AM scan integration, Stage
D remote heap resolution, and the local multi-instance readiness bundle are
reviewed.

Packet 30752 adds the first true local multi-instance timing proof for C1:
`scripts/run_spire_multicluster_transport_overlap_pg18.sh` starts one
coordinator plus two separate remote PG18 clusters, resolves conninfo through
test-only secret names, and calls the production async transport adapter. The
packet artifact records the fast remote completing at 3 ms while the
deliberately slow remote completes at 304 ms. This closes only the
transport-overlap evidence gap; endpoint, epoch, lifecycle, degraded-mode, and
remote heap fixtures remain Stage D/E work.

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

C1 uses per-query connect / per-dispatch close. This matches the current
diagnostic executor lifetime, gives C2 a simple cancellation and cleanup target,
and avoids introducing pool invalidation rules before strict/degraded failure
semantics are in place. Bounded connection reuse or a pool may land only as a
later measured optimization with explicit invalidation triggers.

The C0-C6 executor contract is transport-neutral. `tokio-postgres` is the C1
adapter implementation because the current `pgrx::pg_sys` surface does not
expose the needed libpq async/pipeline entry points; it is not a permanent
protocol commitment. The adapter creates a fresh current-thread runtime per
query, keeps the feature footprint to `rt`, `time`, and `net`, and should add
Tokio features only when a later C2/C5 requirement proves they are needed.

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
If cancellation reaches a dispatch after compact-candidate receive has already
validated and retained a batch, the transition clears that batch and reports
`remote_executor_cancelled` / `local_query_cancelled`; only a later non-cancelled
`CandidateReceiveReady` dispatch may enter compact merge.

Remote statement timeout remains remote-owned and should surface as a remote
timeout category, distinct from local cancel and local statement timeout.

As of 2026-05-10, the first C2 code slice has a narrow remote-cancel primitive:
the `tokio-postgres` adapter keeps a cancel token for each in-flight remote
query, a deterministic test trigger can request remote cancellation, and
executor state maps `local_query_cancelled` outcomes to global
`remote_executor_cancelled` dispatch cleanup. The production PostgreSQL
interrupt bridge now polls backend `InterruptPending` / `QueryCancelPending`
while remote work is in flight and maps that signal to the same adapter cancel
token. Statement-timeout provenance is now split from user/query cancellation
by polling `get_timeout_indicator(STATEMENT_TIMEOUT, false)` without resetting
PostgreSQL's indicator; timeout-triggered local cancellation reports
`local_statement_timeout`, while other local query-cancel flags report
`local_query_cancelled`. Packet 30753 extends the cancellation cleanup proof to
the production async transport and compact-candidate receive adapters: both
paths acquire global/per-node advisory governance permits before remote
connection work, release those permits when local cancellation closes the
attempt, and expose saturated slots as `remote_executor_overload` instead of
leaking raw advisory-lock errors.

The C5 AM bridge must choose one cancellation source and test it under a real
backend, not only under deterministic test triggers. Acceptable shapes are:
the AM scan poll loop observes PostgreSQL interrupt state
(`CHECK_FOR_INTERRUPTS()` / `QueryCancelPending`) and calls the per-query
executor cancel handle, or a backend-owned callback raises a signal that a
watcher task maps to the same cancel handle. Either way, cancellation is
query-wide, every remote attempt receives a fresh cancel token, and a cancelled
token must not be reused across retry attempts.

Verification:

- PG18/local fixture proving a local cancel releases global and per-node
  governance slots.
- PG18/local fixture proving saturated production governance slots fail before
  conninfo parsing or socket open.
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

As of 2026-05-10, the first C4 state slice exists in the production executor:
transport, secret-resolution, and compact-candidate receive failures can move a
dispatch to `degraded_skipped` under degraded mode, summaries expose the skip
count and first skip category, and compact merge ignores skipped dispatches
while consuming only `CandidateReceiveReady` batches. AM-boundary policy and the
full remote fault matrix still need to land before C4 is complete.

The C5 AM boundary must make strict/degraded mode a single per-query executor
input, not an ad hoc per-dispatch flag. The source must be named explicitly
(session GUC or query option), validated before dispatch admission, and threaded
unchanged into transport, candidate receive, compact merge, and Stage D heap
resolution. When degraded mode returns SQL-visible results, diagnostics must
show that the result set is partial by naming skipped nodes, or at minimum
reporting skipped-node count plus first skip category.

As of 2026-05-10, the first AM-boundary policy source is pinned as the session
GUC `ec_spire.remote_search_consistency_mode`, an enum with `strict` and
`degraded` values and default `strict`. The first proof surface is
`ec_spire_remote_search_production_executor_session_summary(...)`, which reads
that GUC and forwards the resulting single consistency mode into production
executor state without accepting a per-call free-form consistency string. C5
must use the same source. A future query-level option may override the GUC for
one statement, but it must not replace the GUC contract or create a second
parallel source inside the executor state.

The production executor state row also carries `consistency_mode_source` and
`consistency_mode` for C5 Rust-side consumers. The existing SQL full-state
surface is already at the pgrx row-width ceiling, so SQL mode attribution stays
on the compact session summary unless a later packet splits the full-state
surface into smaller views.

Before C5 dispatch planning, callers can ask
`ec_spire_remote_search_production_policy_summary(...)` or the session-GUC
variant `ec_spire_remote_search_production_policy_session_summary(...)` whether
the requested consistency mode matches the active published epoch. A mismatch
is a publisher-side guarantee violation and remains fail-closed even when the
session requested degraded mode; the diagnostic category is
`consistency_mode_mismatch`, not a generic planning error.

Verification:

- Fault matrix for auth or certificate failure, connection reset, remote
  backend termination, remote timeout, local cancel, network partition,
  endpoint version skew, stale epoch, fingerprint mismatch, and missing remote
  index.

As of 2026-05-10, the dry SQL-visible
`ec_spire_remote_search_production_fault_matrix()` surface pins the C4 policy
matrix before C5 consumes it. It includes executor governance overload,
transport, secret, remote/local statement timeout, backend termination,
remote/local query cancellation, candidate validation, endpoint identity,
protocol/extension version, stale/served epoch, `consistency_mode_mismatch`,
and reserved Stage D remote heap-resolution categories. Most remote-node
failures fail closed in strict mode and become node skips in degraded mode;
local cancellation and local statement timeout remain query-wide cancellations
in both modes; consistency mode and requested-epoch mismatches fail closed in
both modes.

C5 must consume this matrix, or a Rust-side equivalent generated from the same
category list, as the AM-boundary source of truth instead of re-encoding
strict/degraded behavior in the scan path. The Stage D heap-resolution rows in
the matrix are reserved category names only until the heap executor emits them;
they are not evidence that remote heap resolution is implemented.

### C5: AM Scan Integration

Wire production remote fanout into the coordinator scan path behind an explicit
readiness gate. The scan path must merge local and remote compact candidates
only after candidate batch validation and must defer final SQL row readiness
until Stage D remote heap resolution is production-correct.

C5 consumes only `CandidateReceiveReady` dispatches. Those dispatches are the
handoff contract into Stage D: they contain already validated compact candidate
batches, the selected PID set used for validation, the origin node, and the
candidate row count that remote heap resolution must account for. C5 must not
re-run compact receive, re-resolve conninfo, or reinterpret failed receive rows
as empty ready batches. `CandidateReceiveFailed` dispatches remain strict
fail-closed candidates until C4 degraded semantics explicitly mark them
skippable.

Until Stage D is implemented, C5 may prove ordered compact-candidate merge, but
final SQL row delivery must surface `requires_remote_heap_resolution`. The
remote heap stage is responsible for reading the ready compact candidates,
fetching heap visibility from each origin node, preserving opaque row locators,
and producing the final row stream only after every required remote candidate is
resolved or explicitly skipped by degraded-mode policy.

Packet `30754` adds the first production AM-scan handoff surface:
`ec_spire_remote_search_production_scan_handoff_summary`. It derives selected
leaf PIDs through the same scan router used by `amrescan`, uses a routing-only
leaf count that does not read remote leaf payloads, runs live production compact
candidate receive for remote dispatches, merges only validated ready compact
batches, and then reports `requires_remote_heap_resolution` instead of returning
remote SQL rows. This keeps the Stage D boundary explicit while proving that the
coordinator scan plan can feed the production remote executor.

Packet `30755` adds the first Stage D heap-visibility proof surface:
`ec_spire_remote_search_production_scan_heap_resolution_summary`. It keeps row
locators opaque at the coordinator, gates remote heap receive on
`CandidateReceiveReady`, asks the origin node to resolve heap visibility under
its PostgreSQL snapshot, exact-reranks visible heap rows, fails strict mode when
an indexed remote locator no longer resolves to a visible heap row, and merges
ready local plus remote heap candidates with the existing deterministic
tie-breaker. This is still a summary/proof surface; moving the heap-resolved
stream into `amrescan` / `amgettuple` remains the final Stage D integration
slice.

Verification:

- One coordinator plus two remote PostgreSQL nodes can return one ordered
  candidate stream.
- If Stage D heap receive is not yet complete, final row delivery reports the
  existing `requires_remote_heap_resolution` blocker rather than pretending
  remote rows are SQL-ready.
- Once Stage D heap receive is ready, the proof surface reports `remote_ready`
  only after origin-node heap visibility succeeds; stale or missing remote heap
  rows report `remote_heap_resolution_failed`.

### C6: Operator And Harness Readiness

Expose production-state summaries for operators and packet capture without
opening extra remote sockets. The dry pipeline entrypoint should stay dry; live
diagnostic probes should remain opt-in.

Surfaces this runbook reads:

- `ec_spire_remote_search_libpq_executor_budget_summary(...)`;
- `ec_spire_remote_search_libpq_identity_cache_summary(...)`;
- `ec_spire_remote_search_production_executor_state_summary(...)`;
- `ec_spire_remote_search_libpq_receive_attempts(...)`;
- `ec_spire_remote_pipeline_steps(...)` and
  `ec_spire_remote_pipeline_steps_live(...)`.

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
- Connection reuse policy after C1: C1 is pinned to per-query connect /
  per-dispatch close; performance readiness needs either bounded reuse or
  measured evidence that connect cost is outside the query hot path.
- Shared identity cache: deferred until a memory cap, lock-order contract, and
  descriptor-write invalidation path are accepted.
