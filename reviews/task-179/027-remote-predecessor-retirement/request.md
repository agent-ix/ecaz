---
agent: codex
role: coder
model: gpt-5
date: 2026-07-11
seq: 01
---

# Review request — Packet 027 remote predecessor retirement

This checkpoint extends T4b recovery across every immutable predecessor owner.

## Commit

- `b29366f27` — recover remote predecessor retirement.

## Behavior

- Recovery locks pending dispositions in predecessor-roster order and loads
  their immutable local/remote transport bindings.
- Each owner receives the exact canonical successor activation and digest via
  local SPI or parameterized pooled libpq transport.
- The coordinator changes only that owner's disposition to `Retired` after its
  acknowledgement succeeds. A partial remote failure leaves already committed
  participant marks replayable while coordinator disposition updates roll back.
- The covering successor advances `Activated` to `Applied` only when no
  disposition remains `Pending`.

## Live evidence

The PG18 three-owner fixture publishes a first physical epoch, builds and
publishes a successor, verifies T4a leaves the decision `Activated`, then runs a
later T4b transaction. All three predecessor generations report `Retired`, the
decision reports `Applied`, and the successor remains the active pointer.

Validation and provenance are in `artifacts/manifest.md`.

## Explicit next work

- Remote retire-decision apply/reclaim remains open.
- Physical remote expansion/materialization and the real three-instance fixture
  remain required.
- Persisted bounded head seeds and 10k/50k/100k measurements remain open.

Leaving this request open for outside review.
