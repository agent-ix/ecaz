---
task: 231
packet: 006-prepared-lock-lifetime
agent: Codex
role: coder
model: GPT-5
date: 2026-08-30
seq: 1
---

# Task 231 prepared-transaction lock-lifetime correction

Status: review-open. Code checkpoint:
`fc4a4292681715d899a80d7df251955b5de6f711`. GitHub ticket: issue #97.

Packet 005's first decision attempt exposed a distributed self-deadlock in the
Packet 004 mutation-lock contract. The 10k control arm completed, but the first
fixed-stride candidate stalled during routed inserts. Owner 2 had an earlier
remote write PREPAREd with a granted `ShareRowExclusiveLock` on its raw node
store, while a later independently prepared backlink RPC for the same
top-level insert waited for that same self-conflicting lock. The coordinator
could not resolve the first prepared transaction until the backlink returned,
so neither side could advance. PostgreSQL's local deadlock detector could not
see the coordinator dependency.

The exact lock evidence and invalid-run disposition are committed at
`reviews/task-231/005-full-scale-decision/artifacts/run/fixed-stride-10k-a-stall-diagnostic.md`.
The partial candidate is not A/B evidence. The final matrix will restart all
arms on one corrected extension SHA rather than reuse the old control.

## Correction

`FixedStrideDmlContext` still acquires `ShareRowExclusiveLock` once and retains
it across every raw append made by that owner-local mutation context. Its new
`Drop` implementation releases only that explicit writer lock when the
context's last raw write is complete, before the surrounding remote
transaction can be PREPAREd. The relation guard's separate `AccessShareLock`
continues to protect the relcache/relation lifetime until its own drop.

This preserves the allocator invariant. The raw relation is non-MVCC and its
physical tail is authoritative: a later writer entering after unlock sees all
physically written extents even if the earlier directory transaction is still
uncommitted. If the earlier transaction aborts, those bytes remain an
unreachable ordinal gap, matching Packet 004's accepted monotonic/gap-preserving
contract. No writer can overlap a tail calculation and raw append because the
explicit lock still spans the complete context.

Construction is also failure-safe: the context is constructed immediately
after lock acquisition, so a metadata-admission error drops it and releases
the explicit lock rather than leaking the refcount until transaction end.

## Regression and validation

The existing two-writer Repeatable Read regression is strengthened. Writer 1
now finishes its raw append and deliberately holds its SQL transaction open.
Writer 2 must finish its own append before writer 1 is permitted to commit;
both then commit with distinct ordinals `1,2`. The old transaction-scoped lock
fails this condition, while the corrected physical-tail critical section
passes without relying on MVCC visibility.

Focused PG18 result: `4 passed; 0 failed`. PG18 library clippy with warnings
denied also passes. Commands, hashes, and isolation details are in
`artifacts/manifest.md`.

Please review the early-unlock safety argument, Drop/error paths, and the
strengthened concurrency regression. Packet 005 remains paused at an invalid
first attempt; after this packet is review-closed, I will rebuild the release
extension at the accepted code checkpoint and restart the entire frozen
10k/50k/100k suite from a fresh precheck.
