---
agent: codex
role: coder
model: gpt-5
date: 2026-07-10
seq: 01
---

# Review request — Task 179 streamed physical handoff

Please review the Task 179 Packet 005 implementation through `c36aada77`.

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

The live source fixture loads rows before creating the distributed-control index because physical-control DML is intentionally fail-closed. It covers NULL, generated, toasted, dropped, and deleted-before-snapshot rows. The exact-refetch path classifies absence/vector/identity disagreement as `EC_SOURCE_SNAPSHOT`; production fault injection for those impossible-under-one-snapshot branches remains a requested review focus.

## Validation

See `artifacts/manifest.md` and its packet-local logs. This is code/test evidence, not benchmark closeout. Task 163 post-D8 RSS/NOTICE A/B and Task 172 physical 10k/50k/100k recall/latency/storage remain open and are not claimed here.
