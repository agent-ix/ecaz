---
agent: codex
role: coder
model: gpt-5
date: 2026-07-10
seq: 01
---

# Review request — Task 179 streamed physical handoff

Please review the Task 179 Packet 005 implementation through remediation commit
`417d169859a1c9e37abdd885bf44cc811c21d14a`.

## Scope

- A fixed 107-byte, versioned, restartable owner-stream SHA-256 state pinned to exact `sha2` 0.11 serialization, with independent fixture decoding and compatibility-matrix coverage.
- Participant `stage` with complete decode/canonical binary-I/O validation before mutation, row-tier + graph + unique-directory writes, journal/progress commit, exact replay, conflict/sequence rejection, and digest-state restoration.
- Participant `seal` with an independent physical rescan, owner/locator/identity validation, graph/row/directory digests and byte totals, canonical 303-byte Ready receipt, final-only Building→Ready mutation, and exact replay.
- Exact source capture under one explicitly registered MVCC snapshot. The source-scan key/identity datums are compared with an exact-TID refetch in a second slot before every non-dropped attribute is serialized through local `typsend` into a PostgreSQL-managed `BufFile`.
- Full eventual-entry 8 MiB preflight before Vamana construction, including maximum graph-degree/code payload.
- Two-pass graph workspace: the first pass streams canonical entries from graph + payload spool to compute per-owner expectations; the second routes the same entries after begin through one bounded buffer per owner.
- Owner routing sends sequence zero for empty owners, advances count/hash only after an exact acknowledgement, and retains one unacknowledged encoded batch unchanged for retry.
- Prior Packet 003/004 P2 fixes: nonowner legacy ambuild privilege regression and publish-decision→build-registration retention FK.

## Review boundaries

This packet owns the handoff/storage machinery. The public `ec_distann_begin_epoch_build` / `ec_distann_build_epoch` operator transactions, durable private binding capture, libpq participant dispatch, publish decision/recovery, and build gate remain Packet 006. The workspace/router uses an acknowledgement callback so Packet 006 can bind local or remote transport without weakening batch memory or replay state.

The live source fixture loads rows before creating the distributed-control index because physical-control DML is intentionally fail-closed. It covers NULL, generated, toasted, dropped, HOT-updated, and deleted-before-snapshot rows. HOT-aware index-TID resolution classifies absence/vector/identity disagreement as `EC_SOURCE_SNAPSHOT`; deterministic fault selectors exercise all three otherwise impossible-under-one-snapshot branches.

## Remediation after outside review

- Participant receive/send and source-capture `typsend` now execute under the locked control/source relation owner with `SECURITY_RESTRICTED_OPERATION`, restricted search path, GUC nesting, and `PgTryBuilder::finally` restoration. A hostile user-owned domain proves stage never inherits the SECURITY DEFINER owner.
- The router now retains one in-place canonical wire buffer per owner. It appends entries directly, patches/finalizes once, retains exact finalized bytes across transport/acknowledgement failure, and enforces aggregate allocated capacity at `8 MiB × roster_count`.
- Physical capture now supplies one registered MVCC scan to `table_index_build_scan`, transfers scan ownership correctly, rejects defensive callback-dead input before datum access, and resolves callback index TIDs through HOT-aware `table_index_fetch_tuple`.
- FR-078, Task 179, and TC-040 now state PostgreSQL's actual supplied-MVCC semantics: recently-dead rows are filtered before callback, and callback TIDs may be HOT roots whose visible member must be resolved through the table AM.
- The PG18 failure matrix snapshots catalog JSON, batch journal, relation counts, and row/graph/directory bytes around wrong-owner, duplicate-existing, malformed, schema/codec, noncanonical receive, oversize, corrupted hash state, post-prepare fault, missing row, and wrong/missing directory failures.

## Validation

See `artifacts/manifest.md` and its packet-local logs. This is code/test evidence, not benchmark closeout. Task 163 post-D8 RSS/NOTICE A/B and Task 172 physical 10k/50k/100k recall/latency/storage remain open and are not claimed here.
