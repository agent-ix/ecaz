---
agent: codex
role: coder
model: gpt-5
date: 2026-07-11
seq: 01
---

# Review request — Packet 026 multi-owner publication recovery

This checkpoint extends T4a recovery from one local participant to the complete
immutable successor binding set.

## Commit

- `e4e6d9b1a` — recover multi-owner publication.

## Behavior

- Recovery loads successor bindings in roster order from the private build
  snapshot and publishes each owner through local SPI or pooled parameterized
  libpq transport.
- Every participant acknowledgement must equal the decided 34-byte epoch
  fingerprint. Any unavailable/mismatched owner fails recovery before the
  coordinator active-pointer swap.
- Exact endpoint replay supports recovery after a strict subset of remote
  participants committed publication.
- Only after all acknowledgements match does the existing conditional pointer
  insert/swap run and clear the durable build gate.

## Live evidence

The PG18 three-owner fixture first exercises and verifies remote abort of an
unpublished build. It then builds a fresh epoch, commits the decision in a
separate transaction, runs recovery, verifies all three generation rows are
`Published`, and verifies the coordinator active pointer names that build.

Validation and provenance are in `artifacts/manifest.md`.

## Explicit next work

- T4b remote predecessor marking and remote retire/reclaim dispatch remain
  open.
- Physical remote expansion/materialization and the real three-instance fixture
  remain required.
- Persisted bounded head seeds and 10k/50k/100k measurements remain open.

Leaving this request open for outside review.
