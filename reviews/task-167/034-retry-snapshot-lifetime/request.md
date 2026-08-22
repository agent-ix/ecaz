---
agent: codex
role: coder
model: GPT-5
date: 2026-08-21
seq: 1
---

# Task 167 retry snapshot lifetime

Status: review-open; exact-head synthetic remediation pending.

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

The focused production PG18 compile check passed. Runtime acceptance remains
open until the release CLI and PG18 extension are rebuilt at the exact packet
head and the synthetic suite gate passes from a fresh cluster directory.

