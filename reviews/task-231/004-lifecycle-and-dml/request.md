---
agent: Codex
role: coder
model: GPT-5
date: 2026-08-29
seq: 1
---

# Task 231 fixed-stride lifecycle and DML checkpoint

Status: review-open. Code checkpoint: `08075a341274f9f76df018f503af912d6d95b0e5`.
GitHub ticket: issue #97.

This checkpoint completes Packet 004's fixed-stride lifecycle and Task 167
mutation integration without changing the legacy graph-heap, covering-sidecar,
or hot/cold branches.

## Mutation contract

- Every mutation re-admits the EFM1 relation metadata and binds its relation
  OID, layout digest, and committed base-node count to the Published Ready
  receipt. The Ready count is an immutable ordinal floor: no DML write may
  rewrite a Published base node.
- Inserts append one complete raw node, then publish its `(vec_id,
  node_ordinal, row_tid, record_version, is_current)` directory row in the
  caller's PostgreSQL transaction. Raw node WAL can survive an abort, while
  the MVCC directory publication cannot; the next writer reuses and overwrites
  that unreachable tail ordinal.
- Replacements, tombstones, and backlink amendments append complete overlay
  nodes. The old raw node and historical directory row remain intact; only the
  old directory row's `is_current` flag and the new directory publication move
  transactionally. Tombstones preserve the exact vector, row locator, search
  code, and adjacency rather than mutating a Published base node in place.
- Overlay allocation holds `ShareRowExclusiveLock` on the raw relation through
  transaction end. It remains compatible with serving `AccessShareLock`
  readers and prevents two owner-local writers from selecting the same tail
  ordinal. The allocator scans every historical directory row, not only the
  current partial-unique surface.
- Mutation reads use the fully verified decoder before deriving a replacement,
  tombstone, or backlink, including directory/node identity and row-locator
  agreement. A backlink preserves both a prior tombstone and the raw node's
  embedded exact vector.
- Participant retirement retains the raw relation while readers may drain.
  Reclaim drops it with the row, graph, and source-map relations; transaction
  rollback restores every relation, and repeated reclaim remains idempotent.

## Focused PG18 evidence

- `test_distann_fixed_stride_dml_append_overlay_and_rollback`: PASS, 1/1. It
  covers base ordinal 0, insert ordinal 1, stable-identity replacement ordinal
  2, tombstone ordinal 3 and idempotent retry, an injected post-raw-write abort,
  reuse of the unreachable ordinal 4 tail, and two backlink overlays allocated
  as ordinals 5 and 6 inside one transaction. It verifies exact vectors,
  locators, tombstone preservation, adjacency, historical rows, and MVCC
  rollback.
- `test_distann_fixed_stride_retire_reclaim_rollback`: PASS, 1/1. It proves the
  raw store survives retirement and failed reclaim, then disappears with the
  other generation relations on successful idempotent reclaim.
- PG18 library clippy with warnings denied: PASS.

The packet-local commands, timestamps, hashes, integrity prerequisite, and key
result lines are recorded in [`artifacts/manifest.md`](artifacts/manifest.md).
These are correctness fixtures, not performance evidence; Packet 005 owns the
mandatory suite-driven 10k/50k/100k A/B, cross-owner runtime/DML measurements,
restart/reopen observation, and the PROMOTE-or-STOP decision.

## Reviewer focus

Please review the raw-WAL-before-MVCC-publication ordering, snapshot visibility
after the transaction-scoped ordinal lock, abort-tail reuse, preservation of
the fixed node's exact vector through amendments, and lock ordering when one
coordinator transaction visits multiple owners. Packet 005 must not start its
decision run until any Packet 003 seq-06 and Packet 004 findings that affect
the measured path are closed.
