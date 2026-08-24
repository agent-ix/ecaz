# Task 234: ec_distann Read RPC Deadline and Cancellation Parity

Status: **implementation and 25-cell PG18 fault matrix complete; packet 003
review-open; outside acceptance and packet 004 closeout pending — production
hardening before Task 228** (2026-08-24). Evidence:
`reviews/task-234/003-pg18-fault-matrix/`.
Priority: P0 distributed-read correctness/operations.

## Why

The common async transport gives expansion, materialization, lifecycle calls,
connection setup, session setup, and prepared-statement setup a client-side
deadline, remote `statement_timeout`, PostgreSQL interrupt polling, and a
best-effort remote cancel token. Five read/control RPCs still bypass that
wrapper with bare awaits:

- sharded physical head search;
- crown-code export;
- gateway-routing export;
- head-shard export; and
- head-shard import.

A stalled owner in any of these calls can hold the coordinator backend beyond
the configured budget. Head search is especially important because it occurs
before traversal and therefore can block every distributed query without
returning even a partial result. FR-081 records this as the Task-214 F9
implementation gap.

## Goal

Route every distributed read/control RPC through one deadline, interrupt, and
remote-cancellation contract. Prove that local query cancellation, local
statement timeout, remote statement timeout, connection stall, and owner
failure terminate within bounded time, release pooled state, and never return
partial or stale results.

## Entry conditions

1. The current `remote_transport.rs` call-site inventory is captured in the
   plan packet, including setup, prepared-statement, head, traversal,
   materialization, gateway, crown, and head-shard calls.
2. Task 167 closeout is available as write-side context, but this task does not
   modify distributed DML transaction semantics; Task 235 owns those calls.
3. Fault validation uses PG18 and the `ecaz dev distann-multicluster` surface;
   any repeated missing fault mode is added to that CLI rather than a one-off
   script.

## Required implementation

### P1 — Unified read await contract

- Replace all bare async query/query-one awaits in the five named RPCs with the
  common bounded await path or a shared typed refinement of it.
- Apply nonzero client deadline and remote `statement_timeout` to every call,
  and retain the existing nonzero connect deadline.
- Poll PostgreSQL interrupts before dispatch, while awaiting, and immediately
  after completion; local cancellation stops dispatch of any later owner work.
- Deliver remote cancellation through the connection's cancel token with a
  bounded delivery attempt. If delivery is not confirmed, evict/drop the
  affected pooled connection and its prepared/session state.
- Preserve deterministic fail-closed aggregation: one failed owner fails the
  whole attempt, no successful sibling response becomes a partial result, and
  no response from the failed attempt is reused after restart.

### P2 — Error and pool-state contract

- Preserve distinct stable internal categories for connect timeout, remote
  statement timeout, local statement timeout/query cancel, remote backend
  termination, and transport reset. Task 237 owns the final SQL-visible error
  vocabulary and counters; this task supplies typed outcomes rather than raw
  strings.
- Define when a timed-out or cancelled connection is reusable. Ambiguous
  transaction/session state must force eviction and clean reconfiguration.
- Keep timeout/cancel handling bounded in owners and memory; do not spawn an
  unbounded cancellation task per RPC.

### P3 — Validation

- Add deterministic PG18 multinode faults for all five RPCs: stalled remote
  statement, local `pg_cancel_backend`, local `statement_timeout`, remote
  backend termination, connection reset, and one sibling owner succeeding
  while another fails.
- Assert elapsed upper bounds with documented scheduling tolerance, absence of
  partial rows, no leaked backend work, no retained stale prepared/session
  state, and successful clean retry where the failure class is retriable.
- Run focused unit/pgrx coverage and the relevant PG18 multicluster fault
  cells. This is hardening work, so tests are required despite the repository's
  default static-review policy.

## Non-goals

- TLS and conninfo-secret productionization; Task 236 owns that transport
  substrate.
- Distributed insert/backlink/tombstone transaction-control statements,
  callbacks, and reaper calls; Task 235 owns them.
- Hedging, degraded completion, or silently skipping a failed owner; Task 209
  owns explicitly labeled degraded behavior.
- BatANN, first-hop fusion, cross-query multiplexing, or a new binary wire
  format.

## Acceptance

1. Structural inspection finds no bare awaited read/control RPC outside the
   reviewed wrapper allowlist.
2. Every named fault terminates within its configured budget and produces no
   partial result or leaked remote work.
3. Cancelled/ambiguous connections are evicted; safe connections retain
   bounded pool/prepared-state reuse.
4. Outside review accepts the PG18 fault matrix and FR-081-AC-6 parity claim.

## Required review packets

1. `reviews/task-234/001-plan/`
2. `reviews/task-234/002-wrapper-and-callsite-parity/`
3. `reviews/task-234/003-pg18-fault-matrix/`
4. `reviews/task-234/004-closeout/`

## References

- FR-079; FR-081 implementation gap F9 and FR-081-AC-6
- NFR-014; NFR-020
- Tasks 209, 214, 228, 235, 236, and 237
- `src/am/ec_distann/remote_transport.rs`
