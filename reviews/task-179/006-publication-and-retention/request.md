---
agent: codex
role: coder
model: gpt-5
date: 2026-07-11
seq: 01
---

# Review request — Packet 006 publication/retention contract

Please review specification commit
`d27cc4ce5f89d6542da946aa0a4252f9b294e6b0` before Packet 006 lifecycle code
is committed.

## Why this amendment was required

The initial Packet 006 implementation audit found that the prior contract could
not recoverably represent a durable T2 candidate, a removed predecessor owner,
or idempotent physical reclaim. It also left session-lock rollback, concurrent
build ownership, and cross-backend scan-token storage under-specified.

## Contract now frozen

- Begin-build retains source `ShareLock` then control
  `ShareRowExclusiveLock` as build-specific session ownership, with
  top/subtransaction cleanup, nonblocking competing-backend rejection, one
  gate-active build, exact private-binding registration digest, and durable
  source/control identity gate.
- Generation descriptor v2 binds the authoritative coordinator UUID. Draft v1
  is rebuild-only and Packet 006 owns the encoder/decoder/domain/golden
  migration before lifecycle consumers use it.
- T2 atomically persists an immutable candidate containing registration,
  build-spec, descriptor, source-snapshot, Ready-receipt-set, and manifest
  bytes/digests/fingerprint under a frozen candidate digest.
- Publication is `Pending → Activated → Applied`: T4a publishes successors and
  commits the conditional active-pointer swap; later T4b marks every
  predecessor-roster owner Retired, including removed owners. Scans use the
  predecessor only before activation and the successor after activation.
- Participant retirement and reclaim use canonical activation/retire decision
  bytes plus digests. Reclaim deletes storage transactionally but leaves an
  immutable status/replay tombstone for the control identity's lifetime.
- The coordinator scan registry is a bounded, database-namespaced PostgreSQL
  add-in shared-memory exact-token table. Collision-free allocated fence IDs
  drive short heavyweight shared registration locks and transaction-exclusive
  retirement locks; no LWLock spans SPI, ERROR, commit, or RPC.
- TC-042 now has explicit registration, candidate, pre/post-swap, removed-owner,
  shared-registry, source/control-gate, and reclaim fault axes. TC-050 owns the
  new frozen digest/descriptor fixtures.

## Boundaries

This is a specification/traceability checkpoint. The worktree contains a
separate uncommitted coordinator draft that is deliberately excluded from this
request. Packet 006 implementation, PG18 fault evidence, physical reads, the
three-instance fixture, and benchmark closeout remain open.

## Validation

See `artifacts/manifest.md`. Quire grammar validation and the DistANN
traceability audit are green; the matrix correctly remains PARTIAL because
runtime implementation/evidence has not landed.
