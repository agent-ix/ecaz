---
id: 30238
title: SPIRE Scan Option Plumbing
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 198976bb
---

# Review Request: SPIRE Scan Option Plumbing

## Summary

This checkpoint adds SPIRE-owned reloption/session-option plumbing without
wiring relation-backed persistence or executable AM callbacks.

- Adds `src/am/ec_spire/options.rs`.
- Registers SPIRE reloptions through `amoptions`:
  - `nlists`
  - `nprobe`
  - `rerank_width`
  - `training_sample_rows`
  - `seed`
  - `pq_group_size`
  - `storage_format`
  - `quantizer` alias for `storage_format`
- Registers session GUCs:
  - `ec_spire.nprobe`
  - `ec_spire.rerank_width`
- Adds resolution helpers for effective `nprobe` and rerank width.
- Maps SPIRE storage-format reloptions to assignment payload formats with
  `auto` resolving to TurboQuant for the current Phase 1 default.
- Wires `ec_spire` AM routine `amoptions` to the new parser.

## Non-Goals

- No AM build/scan callback execution.
- No relation-backed partition-object persistence.
- No heap exact-rerank implementation.
- No PQ-FastScan model persistence or scorer binding.

## Review Focus

- Whether `auto -> TurboQuant` is the right Phase 1 default for SPIRE
  assignment payloads.
- Whether SPIRE should mirror `ec_ivf` option names exactly at this stage or
  defer some reloptions until the callbacks consume them.
- Whether `rerank_width = 0` should mean full-frontier rerank for SPIRE, matching
  the helper semantics, or be reserved until AM scan execution lands.
- Whether exposing `pq_fastscan` as a reloption before grouped-PQ metadata lands
  is acceptable given scorer/build helpers still fail explicitly.

## Validation

- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 152 passed, 0 failed
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emitted the existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`.
