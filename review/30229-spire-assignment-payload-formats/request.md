---
id: 30229
title: SPIRE Assignment Payload Formats
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 8dd77505
---

# Review Request: SPIRE Assignment Payload Formats

## Summary

This checkpoint makes SPIRE assignment-row payload formats explicit in code and
in the Phase 0 design note.

- Adds payload-format tags:
  - `NONE = 0`
  - `TURBOQUANT = 1`
  - `PQ_FASTSCAN = 2`
  - `RABITQ = 3`
- Rejects unknown payload formats at assignment row encode/decode.
- Rejects scored leaf and insert-delta rows when `payload_format = NONE`.
- Rejects scored rows with empty payload bytes.
- Keeps delete delta rows constrained to `NONE`, zero `gamma`, and empty
  payload bytes.
- Documents the payload-format values in
  `plan/design/spire-phase0-partition-object-storage.md`.

## Non-Goals

- No quantizer binding or payload-length validation per format.
- No relation-backed persistence.
- No AM scan execution.

## Review Focus

- Whether these numeric tags should be mirrored in ADR-049 or remain in the
  Phase 0 storage-design note until persistence freezes the wire format.
- Whether tombstoned/stale non-scored leaf rows should be further constrained
  to `NONE` immediately, or left permissive for compaction follow-up.
- Whether `PQ_FASTSCAN` should be named more generally as grouped-PQ in the
  SPIRE row format.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 138 passed, 0 failed
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emitted the existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`.
