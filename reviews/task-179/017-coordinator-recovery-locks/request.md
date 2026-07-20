---
agent: codex
role: coder
model: gpt-5
date: 2026-07-11
seq: 01
---

# Review request — Packet 017 coordinator recovery locks

Addresses the lock-order, fail-fast, snapshot-recapture, and T4 candidate-validation
findings carried by packets 009, 011, 012, and 013.

## Commit

- `3c24daf4` — enforce coordinator recovery lock order and candidate validation.

Artifacts and provenance are recorded in `artifacts/manifest.md`.

## Lock ownership and ordering

- `build_epoch`, `decide_epoch_publish`, `recover_epoch_publish`, and
  `abort_epoch_build` now release the SQL argument's short control lock, resolve
  the source/control identity under a preflight AccessShare lock, then acquire
  source `ShareLock` before control `ShareRowExclusiveLock`.
- Both relation acquisitions are conditional. A live owner or conflicting source
  operation returns `EC_BUILD_BUSY` instead of waiting.
- After acquisition, every endpoint reopens the control and revalidates the
  logical UUID and source OID before locking the registry/registration state.
- A replacement backend may inspect/recover/abort durable state, but
  `build_epoch` refuses to capture a new source snapshot under an old build id.
  Exact candidate replay remains allowed and now reconstructs the full candidate
  rather than trusting the stored digest column.

## Recovery digest chain

- Every T4 publish-recovery invocation loads and validates the complete canonical
  build candidate before reading active-pointer state, including the already-active
  replay path.
- Decision candidate digest, fingerprint, manifest digest, and manifest bytes are
  compared against the validated candidate before participant publication or
  pointer mutation.
- Already-Applied decision replay and already-active recovery replay schedule the
  reacquired session locks for release after commit, avoiding a replay-only lock
  leak.

## Regression coverage

`test_distann_build_lock_recovery_guards` uses two real PostgreSQL backends and
proves:

1. the competing backend receives `EC_BUILD_BUSY` while the owner is live;
2. after owner exit it receives `EC_BUILD_STATE` rather than recapturing a new
   snapshot under the durable build id; and
3. the allowed replacement-backend abort reacquires in normative order, commits,
   releases both session locks, and permits source cleanup.

The existing two-epoch test also passes through first activation and successor
predecessor-CAS recovery with the new validation path.

## Still open

- Packet 012 P2-1's single-node SPI fixture still needs conversion to explicit
  build/decide/recover transactions, plus crash-window fault injection between
  durable decide and recovery.
- Packet 007 P2-A/P2-B utility permission/lock-level and ATTACH PARTITION gate
  findings remain open.
- Advisory-lock single-flight, T3 topology re-verification, retirement, physical
  generation reads, the real three-node fixture, and the 10k/50k/100k A/B gate
  remain later Task 179 slices.

Leaving this request open for outside review.
