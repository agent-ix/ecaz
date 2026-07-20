---
agent: codex
role: coder
model: gpt-5
date: 2026-07-11
seq: 01
---

# Review request — Packet 021 local predecessor retirement recovery

This checkpoint completes the single-local T4b transition after a successor
epoch has durably reached T4a activation.

## Commit

- `b75f73a06` — recover local predecessor retirement.

## Recovery behavior

- A replay that observes the successor already active and its covering publish
  decision still `Activated` locks the pending predecessor disposition rows.
- Recovery rejects a remote predecessor explicitly; this checkpoint only
  covers the single-local topology currently accepted by epoch build.
- The participant validates and applies the successor activation before the
  coordinator records the exact activation digest on each disposition.
- The covering publish decision advances to `Applied` only when no predecessor
  disposition remains `Pending`.
- Replays after `Applied` are idempotent and return the active fingerprint.

## Live evidence

The PG18 multi-epoch fixture now preserves the T4a boundary after the first
recovery transaction, then invokes recovery again and verifies the predecessor
generation is `Retired`, its disposition is `Retired`, and the successor
decision is `Applied`.

Validation and provenance are in `artifacts/manifest.md`.

## Explicit next work

- A canonical retire decision, scan-token fence, and physical relation reclaim
  remain required after the logical predecessor-retired acknowledgement.
- Remote participant dispatch and the real three-node fixture remain open.
- Required 10k/50k/100k recall, latency, and storage A/B evidence remains open.

Leaving this request open for outside review.
