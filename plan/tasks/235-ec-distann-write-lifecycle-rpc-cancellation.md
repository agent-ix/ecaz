# Task 235: ec_distann Write and Lifecycle RPC Cancellation Hardening

Status: **proposed — Task 167 is review-closed; ready before Task 228**
(updated 2026-08-25).
Priority: P0 distributed-write correctness/recovery.

## Why

The main remote physical insert/backlink/tombstone endpoint call uses the
bounded async await wrapper, but its surrounding transaction-control and intent
operations do not uniformly do so. Direct `BEGIN`, session `SET`, `PREPARE
TRANSACTION`, `COMMIT`/`ROLLBACK`, intent inserts/updates, transaction callbacks,
and prepared-transaction reaper calls have different synchronous/asynchronous
paths and cleanup behavior.

Applying an ordinary query timeout mechanically is unsafe here: after a lost
acknowledgement the coordinator may not know whether an owner prepared,
committed, rolled back, or merely lost the connection. Hardening therefore
requires an explicit per-phase outcome and recovery contract, not just wrapping
every call in `tokio::time::timeout`.

## Goal

Make every ec_distann distributed write, lifecycle, publication, retirement,
abort, and prepared-transaction recovery call bounded and cancellation-aware
without converting ambiguous remote outcomes into guessed commit/rollback
decisions. Preserve the durable intent/replay fences and give operators a
deterministic recovery action for every uncertain phase.

## Entry conditions

1. Task 167 is review-closed and its insert/replacement/delete, intent, 2PC,
   saturation, and reaper disposition is the baseline.
2. The plan packet inventories every remote statement by phase: connection and
   session setup, begin, endpoint mutation, prepare, intent update, local
   callback, decision application, rollback cleanup, lifecycle/publish, and
   reaper inspection/action.
3. The failure taxonomy distinguishes a remote error from timeout before send,
   timeout after send, lost response, local PostgreSQL cancel, backend death,
   and process exit.

## Required implementation

### P1 — Phase-aware bounded transport

- Give every remote write/lifecycle statement a nonzero connect and statement
  deadline plus PostgreSQL interrupt handling, including transaction-control
  and intent statements currently outside the common async wrapper.
- Stop dispatching new remote work after local cancellation. Attempt bounded
  cancellation only where doing so cannot obscure a durable decision.
- Classify each failed phase as definitely not applied, definitely applied, or
  outcome unknown. Never infer commit or rollback from timeout/connection loss.
- Evict connections with ambiguous transaction state. A pooled session must
  not be reused until transaction state is known clean.

### P2 — 2PC, callback, and recovery safety

- Preserve write-ahead intent ordering and the Task-167 prepared-GID identity.
  The coordinator must have enough durable state to reconcile a lost prepare,
  precommit-intent, commit-prepared, or rollback-prepared acknowledgement.
- Keep PostgreSQL transaction callbacks bounded and non-panicking. Callback
  failure must leave an operator-visible recovery record rather than silently
  discarding the unresolved owner action.
- Make replay/reaper operations idempotent across repeated timeout, process
  crash, and partial-owner completion. A live coordinator XID remains fenced;
  recovery never guesses abandonment from age alone.
- Apply the same contract to build/handoff/publish/retire/abort participant
  calls where they share this transport, without changing generation-state
  authorization or recovery semantics.

### P3 — Fault and operational evidence

- Exercise cancel/timeout/connection-loss at each transaction boundary:
  before mutation, during endpoint mutation, after mutation/before prepare,
  after prepare/before acknowledgement, after precommit intent, and during
  commit/rollback prepared.
- Include coordinator backend termination, owner backend termination, restart,
  duplicate recovery, one-owner partial completion, intent-row loss detection,
  and prepared-slot saturation/readiness hints.
- Assert source and owner row/graph/directory state, intent rows, prepared xacts,
  tombstones, retry outcome, and operator status after every fault.
- Store focused PG18 pgrx and multicluster logs in the task packet.

## Non-goals

- Changing the logical DML routing, graph-neighborhood update algorithm, or
  Task-167 mutation semantics.
- Automatically committing or rolling back an outcome-unknown transaction.
- Background autonomous reaping; v1 recovery remains operator-driven.
- Read-query hedging/degradation or Task-209 semantics.
- Optimizing build/publish throughput; this task only makes its communication
  and recovery bounded/correct.

## Acceptance

1. Every remote write/lifecycle statement is in the reviewed phase inventory
   and has a bounded, explicit outcome contract.
2. No timeout/cancel cell exposes a partial logical mutation, reuses an
   ambiguous connection, or loses the information required for recovery.
3. Repeated operator recovery converges idempotently after every injected
   failure boundary.
4. Outside review accepts the PG18 2PC/lifecycle fault matrix and NFR-014
   operational readiness evidence.

## Required review packets

1. `reviews/task-235/001-plan-and-phase-inventory/`
2. `reviews/task-235/002-bounded-write-transport/`
3. `reviews/task-235/003-2pc-lifecycle-fault-matrix/`
4. `reviews/task-235/004-operator-recovery-closeout/`

## References

- FR-078, FR-082, FR-083, FR-087
- NFR-014 and NFR-020
- Tasks 167, 179, 214, 228, 234, and 236
- `src/am/ec_distann/remote_transport.rs`

