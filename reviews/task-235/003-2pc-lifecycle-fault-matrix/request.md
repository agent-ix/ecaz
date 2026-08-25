---
task: 235
packet: 003-2pc-lifecycle-fault-matrix
agent: Codex
role: coder
model: gpt-5
date: 2026-08-25
seq: 01
---

# Task 235 PG18 2PC and lifecycle fault matrix

Please review code and evidence checkpoint
`b871d5481376df87c60ae486d68bb78519944c21`.

The final release+`pg_test` three-node PG18 matrix ran over Task 236's
verify-full mutual-TLS transport with client-certificate authentication and
plaintext rejection. Extension preflight matched the clean repository head on
all nodes. The fixture passed exactly 23 scenarios and emitted 107 records:
eight lifecycle replay cells, one operator status-unavailable cell, and
fourteen write/recovery cells.

The lifecycle matrix injects a lost acknowledgement after each production
handoff/publish/retire/cancelled-reclaim participant operation. It asserts the
mixed participant state, drives the existing recovery API, verifies the final
participant and coordinator decision state, repeats recovery idempotently, and
checks reclaimed relation residue. Build handoff begin/stage/seal recovery uses
the intentional `abort_then_new_build` contract because a new backend must not
silently recapture a different source snapshot for the same build identity.

The write matrix covers clean commit; failure before mutation; failure after
endpoint mutation; owner backend death during mutation; coordinator death
after prepare followed by immediate owner crash/restart; lost precommit-intent,
commit-prepared, and rollback-prepared acknowledgements; one-owner partial
completion; missing intent detection; full prepared-slot saturation; and owner
death during routed tombstone application. Every cell asserts source rows,
source-map rows, owner graph/current/row state, directory validity, prepared
transactions, nonterminal intents, and the required retry result. Duplicate
recovery always emits zero actions.

The matrix exposed and closed three concrete defects while becoming
secure-current:

- cancelled-generation reclaim compared a coordinator audit UUID with a
  participant-local logical UUID; recovery now validates the coordinator UUID
  against the generation descriptor and the participant UUID/node against the
  descriptor roster;
- Task 235 reaper/operator fixture sessions replaced secure roster entries with
  plaintext socket conninfos; every recovery path now preserves the selected
  secure fixture; and
- the owner crash cell restarted PostgreSQL through a plaintext-only helper;
  it now restores SSL, both secure listeners, and secret-backed conninfos.

Focused `remote_transport::tests` passed 19/19. The packet also contains the
clean-head CLI build, SSL PG18 release install, full final console, compact
matrix, exact commands, hashes, timestamps, and fixture provenance.

Please focus review on lost-ack decision authority, coordinator/owner crash
recovery, lifecycle replay authorization, the cancelled-generation identity
fix, secure transport preservation during recovery/restart, and whether the 23
cells fully cover Task 235 acceptance items 1--3.
