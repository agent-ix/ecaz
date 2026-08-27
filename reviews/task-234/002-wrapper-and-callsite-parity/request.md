---
task: 234
packet: 002-wrapper-and-callsite-parity
agent: Codex
role: coder
model: gpt-5
date: 2026-08-24
seq: 01
---

# Task 234 unified read-await and call-site parity

This packet requests review of code checkpoint `e2c582cff`. It routes all five
inventory gaps—physical head search, crown-code export, gateway-routing export,
head-shard export, and head-shard import—through the bounded read-await
contract. Transactional insert, backlink, tombstone, prepared-transaction, and
reaper calls remain in Task 235's explicit allowlist.

The read contract now checks PostgreSQL interrupts before dispatch, while
awaiting, and immediately after completion. A client deadline performs one
bounded remote cancel-token delivery attempt, then treats the session as
ambiguous and evicts its pooled client, driver task, prepared statements,
session identity, timeout state, and cached physical query digest. Local
interrupts retain the existing whole-pool clear before PostgreSQL raises.

Typed internal outcomes distinguish connect timeout, client read deadline,
remote statement timeout, local query cancel/statement timeout, remote query
cancel, remote backend termination, and transport reset. A completed remote
statement timeout or query cancel is safe to reuse; client deadlines, local
interrupts, backend termination, and transport reset force eviction. Task 237
still owns the final SQL-visible vocabulary and counters.

All physical and logical multi-owner read batches now normalize one owner
failure into the first request-order error for every result. Successful sibling
rows therefore cannot be consumed as a partial result. Single-owner
traversal-replica and head-shard calls apply the same eviction rule. Connection,
remote-timeout setup, session identity, and statement preparation errors are
also typed and evicted when completion is ambiguous.

Focused PG18 library tests pass both the normal and benchmark-feature physical
read shapes, including deterministic uniform failure, successful request-order
preservation, typed pool disposition, bounded await, connection redaction, and
existing response coverage. The structural scan is packet-local. The full
multinode fault matrix remains packet 003 work; this packet does not claim the
outside-review acceptance gate.

No repository-wide formatter was invoked. The functional commit contains only
`expand_error.rs` and `remote_transport.rs`; this request and its validation
artifacts are a separate commit.

Please review interrupt race boundaries, timeout cancel delivery, error
classification, safe-versus-ambiguous reuse, fail-closed batch normalization,
and the Task 235 allowlist boundary.
