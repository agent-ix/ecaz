---
task: 238
packet: 001-retry-snapshot-uaf
agent: Codex
role: coder
model: gpt-5
date: 2026-08-27
seq: 03
---

# Task 238 current-main reconciliation and closeout request

Please review Task 238 for closeout on the current mainline.

## Corrected chronology

The original task and seq-01 feedback were authored from a stale Task 222
planning branch and said the lifetime fix still had to land on main. Current
history shows that the equivalent fix, `15f7fcf5f` ("Retain DistANN retry
snapshots across traversal"), had already reached main through Task 167 PR #77
before Task 238 was filed. Commit `010a0accc` independently fixed the stale
planning branch.

The missing current-main deliverables were therefore deterministic regression
coverage, durable packet evidence, and canonical task/index bookkeeping—not a
second copy of the production fix.

## Current integration

Commit `7d4103885`, based on merged main `3c81319a3`, ports the previously
verified forced-retry regression block onto exact current main and keeps it
self-contained for both callers of the three-owner fixture. It also makes the
shared PG18 loopback conninfo explicitly request `sslmode=disable`, the only
plaintext mode allowed by the production Task 236 policy.

The Task 234 read-RPC and Task 235 write-fault `pg_test` controls are already
present on current main. This checkpoint therefore adds only the missing
Task 238 regression and the loopback test conninfo correction; it does not
duplicate or remove either accepted hardening surface.

No production TLS or runtime behavior is weakened by either repair.

## Evidence and acceptance reconciliation

- Historical same-tree before/after proof remains in this packet: without the
  guard-lifetime change the backend receives SIGSEGV; with it the test passes.
- Exact-current-main projection/forced-retry caller: 1 passed, 0 failed,
  129.15s (`artifacts/pg18-merged-main-projection-contract.log`).
- Exact-current-main sibling handoff caller: 1 passed, 0 failed, 67.86s
  (`artifacts/pg18-merged-main-sibling-handoff.log`).
- `cargo fmt --all --check` passes (stable-rustfmt nightly-option warnings only).

The pre-fix blast radius includes backend crashes and potentially wrong
visibility answers when freed snapshot memory remains plausible. Ordinary
three-owner reads can reach the retry path without external concurrency. Task
222's published matrices ran with the lifetime fix present, so those results
are not exposed.

Please rule on the one historical acceptance deviation: the equivalent fix
landed through Task 167 before Task 238 existed, rather than through a later
standalone Task 238 PR. The requested disposition is ACCEPT/complete because
the code, deterministic regression, blast-radius disclosure, and current-main
verification now all exist; no benchmark gate applies to this correctness-only
task.
