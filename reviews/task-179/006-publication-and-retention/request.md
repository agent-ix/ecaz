---
agent: codex
role: coder
model: gpt-5
date: 2026-07-11
seq: 01
---

# Review request — Packet 006 build-gate foundation

Please review the corrected Packet 006 contract and first implementation
foundation:

- contract amendment: `c152ef9751747fabf58b75f89207ab1eba4e6b0`
- immediate predecessor-cleanup traceability: `004e520c3`
- implementation foundation: `54d09c177`

This supersedes the earlier spec-only request at `d27cc4ce5`; Claude's
`2026-07-11-02-reviewer.md` CHANGES REQUIRED disposition was treated as the
controlling outside review.

## Contract corrections

- A permanently unavailable predecessor can no longer wedge publication:
  each immutable predecessor binding terminates as exact `Retired` or explicit
  audited `Abandoned`; audit insertion and Pending→Abandoned CAS are atomic and
  exact replay returns stored bytes/time.
- `Applied` means every predecessor binding has a truthful terminal
  disposition, not that an unreachable participant reclaimed its local orphan.
- Normal/forced predecessor retirement requires its covering successor
  decision to be Applied. Retire decisions carry the exact abandoned
  ordinal/audit-digest set and skip those bindings during reclaim recovery.
- Scan-token liveness, dead-token reaping, fence operation references, safe
  dropped-UUID fence recycling, candidate re-verification, canonical roster
  encoding, READ COMMITTED rejection, and TC-050 mappings are now explicit.

## Foundation implemented

- `ec_distann_begin_epoch_build` persists one durable source/control build gate
  and exact private-binding digest, with source→control→registry→registration
  ordering and nonblocking competing-backend rejection.
- Session ownership is correct across nested subcommit/subabort, top-level
  abort, terminal replay, outer commit, backend exit, DROP, and REINDEX.
  Commit-only release intents are tracked by subtransaction ID so a rolled-back
  savepoint cannot release parent ownership later.
- Descriptor v2 binds the authoritative coordinator UUID and has exact offsets,
  independent fixture coverage, rebuild-only v1 rejection, and an updated
  writable-format matrix. Registration v1 remains correctly classified as a
  canonical digest preimage rather than a persisted readable format.
- SQL freezes build registration/candidate/publication/retirement/reclaim state,
  exact predecessor dispositions, abandon audits, Applied-covering-successor
  FKs, and immediate predecessor-chain integrity.
- Destructive cleanup removes a publication chain successor/leaf-first,
  detects cycles, and is transactional. REINDEX identifies the prior control
  from its durable registry row rather than already-replaced block-0 storage.
- Revoked SECURITY DEFINER bridges validate exact control metadata/UUID for
  ordinary-owner cleanup and replacement-registry initialization. AM-owned
  relation deletion is marked internal; the global SPIRE sql_drop cleanup
  trigger now runs under a fixed trusted SECURITY DEFINER boundary so ordinary
  DistANN DROP does not fail on SPIRE's revoked catalogs.

## Evidence and review history

The expanded PG18 tests cover:

- same-backend reacquisition after top-level abort;
- terminal replay and destructive REINDEX inside a rolled-back savepoint,
  followed by outer commit and a competing backend that must remain busy;
- terminal commit release, repeated REINDEX, backend-exit reacquisition, and
  same-backend DROP;
- ordinary-owner DROP and REINDEX with all installed event triggers enabled;
- committed and aborted cleanup of a three-decision predecessor chain, exact
  hidden-relation removal, UUID replacement, and full rollback restoration.

Two final read-only audits reported CLEAN after these runs. See
`artifacts/manifest.md` for exact-SHA logs.

## Still open in Packet 006

This request is for the build-gate/schema/format foundation, not full Packet
006 closure. Candidate sealing, T3 decision creation, T4a/T4b publication and
recovery, abandon endpoint execution, DML/utility gate hooks, the shared scan
registry/fence implementation, retirement/reclaim RPCs, and the complete
TC-042 fault matrix remain subsequent code checkpoints. Packet 007 read-path,
Packet 008 physical three-instance validation, and Packet 009 benchmark
closeout also remain open.
