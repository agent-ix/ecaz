---
id: 30254
title: SPIRE Snapshot Scan Preparation
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: cc0484b7
---

# Review Request: SPIRE Snapshot Scan Preparation

## Summary

This checkpoint adds the helper that future relation-backed `amrescan` wiring
can call after it has loaded a published SPIRE snapshot and local object store.

- Adds `SpirePreparedScanCandidates` with resolved scan plan plus ranked
  candidates.
- Adds `prepare_single_level_snapshot_scan_candidates()` to derive leaf count
  from the loaded root object, resolve scan options, route/query-score/rerank,
  and return the prepared candidates.
- Keeps relation-backed snapshot/object loading unwired.

## Non-Goals

- No PostgreSQL relation persistence reads.
- No heap exact rerank implementation.
- No direct `amrescan` call into this helper until the snapshot loader exists.

## Review Focus

- Whether this is the right boundary between loaded snapshot state and live
  scan opaque population.
- Whether option resolution belongs inside this helper once the root leaf count
  is available.
- Whether the exact-rerank callback remains flexible enough for heap-backed
  scoring later.

## Validation

- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 164 passed; 0 failed
- `cargo fmt`
  - Completed with the repository's existing stable-rustfmt warnings for
    nightly-only `imports_granularity` and `group_imports`.
- `cargo fmt --check`
  - Completed with the same rustfmt warnings.
- `git diff --check`
- `git diff --cached --check`
