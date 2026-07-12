---
agent: codex
role: coder
model: gpt-5
date: 2026-07-11
seq: 01
---

# Review request — Packet 019 scan-token RAII

Closes the scan-registry P2-1/P2-2 prerequisites carried by packet 006 before
the physical Published-generation read path consumes scan tokens.

## Commit

- `230459ce` — RAII scan token guard and fingerprint-scoped live count.

## Changes

- `ScanTokenGuard::register` generates an RFC4122-v4 token with PostgreSQL's
  strong RNG, takes the logical-index shared registration fence, registers the
  exact `(database, logical UUID, fingerprint, token)`, and owns its release.
- `Drop` eagerly releases on normal executor teardown and error unwinding;
  abrupt backend death remains covered by the existing owner-generation reaper.
- `live_token_count_for_fingerprint` reaps dead owners and counts only the exact
  fingerprint, preventing retirement of epoch A from being blocked by scans on
  epoch B under the same logical index.
- The ignored preloaded-shmem integration drill now asserts guard scope changes
  the exact count `1 → 2 → 1 → 0`; the pure registry test independently proves
  fingerprint scoping.

Validation and provenance are recorded in `artifacts/manifest.md`.

## Still open

The guard is intentionally not yet wired into CustomScan in this packet. The
next read-path checkpoint must resolve the active Published generation and hold
one guard across graph search and frozen row-tier materialization.

Leaving this request open for outside review.
