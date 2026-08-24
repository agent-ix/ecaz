---
task: 234
packet: 003-pg18-fault-matrix
agent: Codex
role: coder
model: gpt-5
date: 2026-08-24
seq: 01
---

# Task 234 PG18 read RPC fault matrix

This packet requests review of Task 234 through head `0f0fef941`. It adds a
`pg_test`-only named endpoint delay, a test-only five-RPC probe and transport
snapshot, and a first-class `ecaz dev distann-multicluster
--read-rpc-fault-matrix` lane. Production builds expose none of the test GUCs
or SQL probes. The CLI lane runs as a bounded post-publish diagnostic and does
not create a one-off shell sweeper.

The final release-extension PG18 run passed 25 of 25 cells: physical head
search, crown-code export, gateway-routing export, head-shard export, and
head-shard import crossed with remote statement timeout, local
`pg_cancel_backend`, local `statement_timeout`, remote backend termination,
and connection reset. Every fault returned an error rather than rows, stayed
inside its elapsed bound, left no matching remote work active, and completed a
positive clean retry in the same coordinator session.

For each of the three fan-out RPCs, the remote-timeout cell records two
successful siblings and one failed owner before normalization. The SQL caller
still received only the failure. This directly proves the no-partial-result
contract instead of inferring it from final row counts.

Pool disposition also matches the implementation contract:

- completed remote statement timeouts retain the pooled session (and prepared
  statement where that RPC has one), then refresh the timeout and retry;
- local cancel and local statement timeout clear all ambiguous pool state;
- remote backend termination and connection reset evict the affected owner,
  retaining successful sibling sessions for fan-out calls; and
- the single export/import callers evict their sole ambiguous session.

The slowest observed failure was a 628 ms connection reset against a 5,000 ms
tolerance. All remote/local timeout and cancel cells completed in 29–507 ms
against a 2,000 ms tolerance. The pre-fault physical topology covered all
2,000 rows with no non-owned or orphaned records, and the ordinary serving
smoke returned 10 rows.

One non-cited intermediate fresh fixture hit a PostgreSQL
`SubTransGetTopmostTransaction` assertion in the ordinary serving smoke before
the matrix began; the same smoke passed in both the preceding and final fresh
fixtures. The manifest records this transparently so review can decide whether
it needs a separate follow-up. It does not weaken or substitute for any of the
25 exit-0 fault cells.

No repository-wide formatter was invoked. Code, harness, CLI, and review
packet changes remain in separate commits; no formatting-only diff is mixed
with functional work.

Please review the pg_test-only boundary, fault determinism, sibling-status
telemetry, safe/ambiguous pool assertions, elapsed tolerances, remote-work
drain checks, and whether the transient pre-matrix PostgreSQL assertion should
be carried to a separate task before packet 004 closeout.
