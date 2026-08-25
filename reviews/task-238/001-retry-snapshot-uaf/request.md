---
task: 238
packet: 001-retry-snapshot-uaf
agent: Codex
role: coder
model: gpt-5
date: 2026-08-25
seq: 02
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

Commit `3b8b872d6` ports the forced-retry regression block to exact current main
and keeps it self-contained for both callers of the three-owner fixture. It
also repairs two test-only integration gaps exposed while verifying the test:

1. Task 236's integration had carried Task 234 read-RPC delay call sites into
   `generation_read.rs` without the `pg_test` GUC definitions/accessor from
   `options.rs`, so every PG18 `pg_test` build failed to compile.
2. The shared PG18 loopback conninfo did not explicitly select
   `sslmode=disable`, so Task 236's secure default correctly rejected the
   plaintext fixture. The helper now requests plaintext explicitly only for
   the loopback Unix-socket fixture.

No production TLS or runtime behavior is weakened by either repair.

## Evidence and acceptance reconciliation

- Historical same-tree before/after proof remains in this packet: without the
  guard-lifetime change the backend receives SIGSEGV; with it the test passes.
- Current-main projection/forced-retry caller: 1 passed, 0 failed, 73.90s.
- Current-main sibling handoff caller: 1 passed, 0 failed, 59.70s.
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
