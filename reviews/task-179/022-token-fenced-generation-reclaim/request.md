---
agent: codex
role: coder
model: gpt-5
date: 2026-07-11
seq: 01
---

# Review request — Packet 022 token-fenced generation reclaim

This checkpoint implements the normal single-local FR-082 retire-decision and
physical-reclaim path after predecessor retirement.

## Commit

- `ce84a8bc2` — fence and reclaim retired generations.

## Retirement decision

- `ec_distann_retire_epoch(regclass, bytea)` validates the v2 fingerprint,
  acquires the logical index's transaction-exclusive retirement fence, rejects
  the active fingerprint, requires an `Applied` covering successor decision,
  locks the exact terminal predecessor dispositions, and observes the exact
  local scan-token count.
- A nonzero count raises `EC_RETENTION_ACTIVE` and creates no decision.
- At zero, the coordinator converts the versioned build-registration roster
  envelope to canonical roster-v1 bytes, constructs and digest-validates the
  immutable normal `DistannRetireDecisionV1`, and commits it as `Pending`.
  It performs no participant reclaim in that transaction.
- Existing canonical decisions replay without mutation.

## Registration and recovery

- Physical scan registration now checks for a committed retire decision while
  holding the same logical-index shared fence used to insert its exact token.
  A committed decision rejects registration before participant access.
- `ec_distann_recover_epoch_retire(regclass, bytea)` independently decodes and
  verifies stored decision identity, applies the participant reclaim endpoint,
  and records the coordinator decision `Applied`.
- Participant apply atomically leaves the immutable Reclaimed tombstone before
  deleting generation catalog/storage relations; exact decision and recovery
  replay succeed from durable evidence.

## Live evidence

The PG18 two-epoch fixture holds a cross-backend predecessor pin and verifies
normal retirement fails with no decision. After releasing it, the fixture
verifies decision-before-drop, post-decision registration rejection, later
physical reclaim, Reclaimed status, generation-row deletion, and exact replay.

Validation and provenance are in `artifacts/manifest.md`.

## Explicit next work

- Recovery currently fails closed on a non-local binding. Remote lifecycle
  dispatch and partial-reclaim recovery remain required.
- The audited forced-retire and predecessor-abandon operator endpoints remain
  open.
- The real three-node fixture, bounded persisted head seeds, and required
  10k/50k/100k measurements remain open.

Leaving this request open for outside review.
