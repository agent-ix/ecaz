---
agent: codex
role: coder
model: gpt-5
date: 2026-07-11
seq: 01
---

# Review request — Packet 028 remote generation reclaim

This checkpoint completes normal retire-decision recovery across remote physical
owners.

## Commit

- `b7c123c47` — reclaim remote retired generations.

## Behavior

- Retire recovery loads the target epoch's immutable participant bindings and
  skips only ordinals explicitly present in the canonical abandoned set.
- Each non-abandoned owner receives the exact stored retire-decision bytes and
  digest through local SPI or parameterized pooled libpq transport.
- Participant apply remains atomic: tombstone insert, physical relation drop,
  and generation-row deletion commit together. Exact replay succeeds from the
  tombstone after partial coordinator/remote failures.
- The coordinator retire decision advances `Pending` to `Applied` only after
  every non-abandoned call succeeds.

## Live evidence

The PG18 three-owner fixture commits a zero-pin retire decision for the
three-owner predecessor after T4b, recovers it, and verifies zero generation
rows, exactly three immutable reclaim tombstones, coordinator decision
`Applied`, and an unchanged active successor.

Validation and provenance are in `artifacts/manifest.md`.

## Explicit next work

- Physical remote expansion/materialization and the real three-instance fixture
  are now the dominant correctness gap.
- Persisted bounded head seeds and 10k/50k/100k measurements remain open.

Leaving this request open for outside review.
