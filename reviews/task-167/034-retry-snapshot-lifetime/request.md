---
agent: codex
role: coder
model: GPT-5
date: 2026-08-21
seq: 1
---

# Task 167 retry snapshot lifetime

Status: review-open; snapshot remediation verified diagnostically, concurrent
2PC gate still failing.

Please review code checkpoint `15f7fcf5f`.

Packet 033's second synthetic run reached healthy release provenance,
three-owner topology, and initial distributed serving, then PostgreSQL aborted
the coordinator backend on
`TransactionIdFollowsOrEquals(xid, TransactionXmin)`. Symbolication placed the
extension frame in `generation_read::lookup_graph_nodes` during a later
traversal round.

The reopened visibility-retry helper returned a registered latest snapshot,
but `GenerationExpander` retained only its raw pointer. The local guard dropped
at the end of `expand_nodes_masked`, leaving the next traversal round to scan
through freed snapshot storage.

This checkpoint makes ownership explicit:

- the helper returns no replacement guard when the original snapshot
  succeeds;
- a genuinely refreshed snapshot is returned as `Some(guard)`;
- both retained and ordinary generation expanders keep that guard alive for
  every subsequent use of its raw snapshot pointer.

The focused production PG18 compile check passed. The release rerun no longer
crashed: it passed initial serving and both full remote-owner materialization
proofs, directly crossing the packet-033 failure boundary. It then failed the
later concurrent 2PC gate: the controlled six-neighbor target remained at six
after the two writer inserts, reverse-edge coverage was 10/24, and the natural
retry attribution count was zero.

That run is diagnostic, not acceptance evidence. Its embedded build head was
`b33af0342-dirty` because packet-local build logs were created before Cargo
captured provenance. The synthetic suite failed, and the 10k/50k/100k steps
were not selected. The next checkpoint must correct the concurrency overlap
and deterministic-target fixture, rebuild from a clean worktree, and pass the
synthetic gate before any real-corpus scale runs.
