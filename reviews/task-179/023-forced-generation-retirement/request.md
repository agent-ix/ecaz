---
agent: codex
role: coder
model: gpt-5
date: 2026-07-11
seq: 01
---

# Review request — Packet 023 audited forced generation retirement

This checkpoint adds the explicit operator override for reclaiming a non-active
Retired generation while coordinator-local scan pins remain.

## Commit

- `1959c3374` — audit forced generation retirement.

## Behavior

- Normal and forced retirement share one decision implementation and the same
  exact logical-index retirement fence.
- `ec_distann_force_retire_epoch(regclass, bytea, text)` still rejects the
  active fingerprint and requires an `Applied` covering successor decision.
- For a non-active Retired fingerprint, the immutable canonical decision stores
  `forced=true`, the exact overridden token count observed under the fence,
  session caller, bounded nonempty reason, and decision timestamp before any
  participant reclaim.
- Exact replay with the same forced mode/reason succeeds from the durable
  decision. A different mode or reason raises `EC_EPOCH_STATE` without changing
  the audit.
- The legacy one-argument metadata-page prototype remains available as an SQL
  overload; the physical endpoint uses a distinct internal symbol.

## Live evidence

The PG18 fixture first proves force-retire rejects the active second epoch. It
publishes a third epoch, pins the now-Retired second fingerprint, commits a
forced decision with overridden count exactly one, reclaims while that pin is
still deliberately live, and verifies exact versus conflicting replay.

Validation and provenance are in `artifacts/manifest.md`.

## Explicit next work

- Binding-specific audited predecessor abandonment remains required.
- Remote build/publication/retirement dispatch and the real three-node fixture
  remain open.
- Bounded persisted head seeds and the required 10k/50k/100k measurements
  remain open.

Leaving this request open for outside review.
