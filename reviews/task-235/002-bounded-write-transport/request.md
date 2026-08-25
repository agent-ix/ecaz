---
task: 235
packet: 002-bounded-write-transport
agent: Codex
role: coder
model: gpt-5
date: 2026-08-25
seq: 01
---

# Task 235 bounded write/lifecycle transport checkpoint

Please review code checkpoint `7584c1bf3fc14569b9bfc7928d6a18e2a15728d5`.

This slice removes the unbounded async gaps around the already-bounded
physical endpoint call. `BEGIN`, session setup, endpoint mutation,
`PREPARE TRANSACTION`, intent record/update, cleanup rollback, standalone
tombstone commit, and every remote build/handoff/publish/retire/abort call now
use one deadline/interrupt/cancel wrapper. Any write-path error evicts the
pooled session. Errors name the phase and classify the result as
`definitely_not_applied` only when PostgreSQL returned an explicit statement
error (or the phase cannot apply a logical mutation); timeout, local cancel,
connection loss, prepare, commit, and lifecycle acknowledgement loss remain
`outcome_unknown` and require replay/recovery.

The blocking post-commit/post-abort callback and operator-reaper connector now
sets a TCP user timeout in addition to connect and server statement timeouts.
The preplanning and precommit intent operations moved to the async bounded
wrapper, so ordinary PostgreSQL cancellation stops new work and evicts the
session.

The important recovery correction is that an owner prepared transaction is no
longer committed merely because an independently committed intent row says
`commit_intended`. A lost precommit-intent acknowledgement can leave that row
behind even if the coordinator aborts. New GIDs carry the coordinator's
epoch-qualified full xid, and the reaper asks coordinator-local
`pg_xact_status(xid8)` for the actual decision:

- `committed` -> `COMMIT PREPARED`;
- `aborted` -> `ROLLBACK PREPARED`;
- `in progress` -> leave fenced; and
- unavailable/truncated status -> stop as operator-required, never infer from
  age or intent state.

Recovery enumerates the union of owner `pg_prepared_xacts` and nonterminal
intent GIDs. This closes the lost-decision-ack window: if prepared resolution
succeeded but its response or terminal audit update was lost, the next run sees
the still-nonterminal intent, confirms the coordinator outcome, and converges
the row to `commit_local`/`rollback_local`. A prepared GID with a missing intent
is also detected and resolved from coordinator status rather than guessed.

The packet-local phase inventory records every call family and the remaining
runtime evidence owed by packets 003/004. This is not a closeout request:
phase-by-phase PG18 multicluster fault injection, duplicate recovery, partial
owner completion, backend termination, restart, and operator-readiness proof
remain required.

Validation at this checkpoint:

- `cargo check --no-default-features --features pg18,pg_test` — pass;
- `cargo test --lib remote_transport::tests --no-default-features --features pg18,pg_test`
  — 17 passed, 0 failed;
- `cargo fmt --all -- --check` and `git diff --check` — pass.

Please focus review on the outcome taxonomy, mandatory eviction, full-xid
identity, `pg_xact_status` decision authority, callback bounds, and whether any
write/lifecycle call family is absent from the inventory.
