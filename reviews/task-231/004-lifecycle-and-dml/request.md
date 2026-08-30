---
agent: Codex
role: coder
model: GPT-5
date: 2026-08-30
seq: 2
---

# Task 231 fixed-stride lifecycle and DML checkpoint

Status: review-open. Code checkpoint: `471bfe372`.
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
  caller's PostgreSQL transaction. Allocation comes from the raw relation's
  physically written tail while holding its mutation lock, not from an MVCC
  directory maximum. Raw node WAL can survive an abort while the directory
  publication cannot, so the next writer advances past the unreachable raw
  extent and leaves a deliberate ordinal gap; raw ordinals are never reused.
- Replacements, tombstones, and backlink amendments append complete overlay
  nodes. The old raw node and historical directory row remain intact; only the
  old directory row's `is_current` flag and the new directory publication move
  transactionally. Tombstones preserve the exact vector, row locator, search
  code, and adjacency rather than mutating a Published base node in place.
- The mutation context acquires `ShareRowExclusiveLock` on the raw relation
  exactly once and retains it through transaction end. It remains compatible
  with serving `AccessShareLock` readers and serializes physical-tail
  allocation without an MVCC directory scan. Packed tails are fully decoded
  before their next slot is admitted; multi-block tails must end on an exact
  extent boundary.
- Mutation reads use the fully verified decoder before deriving a replacement,
  tombstone, or backlink, including directory/node identity and row-locator
  agreement. A backlink preserves both a prior tombstone and the raw node's
  embedded exact vector.
- Participant retirement retains the raw relation while readers may drain.
  Reclaim drops it with the row, graph, and source-map relations; transaction
  rollback restores every relation, and repeated reclaim remains idempotent.

## Focused PG18 evidence

- `test_distann_fixed_stride_dml_append_overlay_and_rollback`: PASS. It
  covers base ordinal 0, insert ordinal 1, stable-identity replacement ordinal
  2, tombstone ordinal 3 and idempotent retry, an injected post-raw-write abort,
  the unreachable ordinal 4 gap, retry at ordinal 5, and two backlink overlays
  allocated as ordinals 6 and 7 inside one transaction. It verifies exact vectors,
  locators, tombstone preservation, adjacency, historical rows, and MVCC
  rollback.
- `test_distann_fixed_stride_repeatable_read_ordinal_allocation`: PASS. Two
  loopback sessions both establish Repeatable Read snapshots before writing;
  the second blocks behind the first transaction and then appends ordinal 2,
  proving snapshot age cannot cause ordinal collision.
- `test_distann_fixed_stride_retire_reclaim_rollback`: PASS. It proves the
  raw store survives retirement and failed reclaim, then disappears with the
  other generation relations on successful idempotent reclaim.
- `test_distann_fixed_stride_stage_seal_receipt_and_topology`: PASS. Together,
  the combined focused run is `4 passed; 0 failed` at `471bfe372`.
- PG18 library clippy with warnings denied: PASS at the original checkpoint;
  the seq-02 change is covered by the focused PG18 build and tests above.

The packet-local commands, timestamps, hashes, integrity prerequisite, and key
result lines are recorded in [`artifacts/manifest.md`](artifacts/manifest.md).
These are correctness fixtures, not performance evidence; Packet 005 owns the
mandatory suite-driven 10k/50k/100k A/B, cross-owner runtime/DML measurements,
restart/reopen observation, and the PROMOTE-or-STOP decision.

## Reviewer focus

Please re-review the four seq-01 findings. The allocator no longer consults an
MVCC snapshot, performs no directory scan, and locks once per mutation context;
the combined PG18 receipt exercises the new concurrent Repeatable Read case.
Packet 005 will measure raw-store bytes before/after its DML workload. Its
decision run remains blocked until Packet 004 and Packet 005 preregistration
are review-closed.
