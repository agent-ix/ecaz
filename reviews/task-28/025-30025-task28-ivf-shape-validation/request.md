# Review Request: Task 28 IVF Shape Validation

Status: open
Owner: coder2
Code checkpoint: 384f559cc2bb9cb8dd78a2faabcc2e5bb517587c
Branch: task28-ivf
Date: 2026-04-25

## Scope

This packet covers the Phase 5 shape-validation checkpoint for the first
`ec_ivf` access method baseline.

Changes in scope:

- Reject `storage_format = 'pq_fastscan'` and `storage_format = 'rabitq'`
  before IVF build, insert, or scan paths proceed.
- Keep `storage_format = 'auto'` and `storage_format = 'turboquant'` as the
  supported first-baseline shapes.
- Validate posting payload length against the canonical quantizer shape during
  build and live insert paths.
- Mark the shape-validation checklist item complete in
  `plan/tasks/28-ivf-access-method.md`.

## Files

- `src/am/ec_ivf/options.rs`
- `src/am/ec_ivf/build.rs`
- `src/am/ec_ivf/insert.rs`
- `src/am/ec_ivf/scan.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

## Validation

PG18-only validation:

- `cargo check --no-default-features --features pg18 --tests`
- `cargo pgrx test pg18 test_ec_ivf_pq_fastscan_storage_is_unsupported`
- `cargo pgrx test pg18 test_ec_ivf_insert_appends_posting_and_updates_stats`
- `git diff --check`

PostgreSQL version: 18.3 via pgrx `pg18`.

No measurement claims are made in this packet.

## Review Focus

- Whether rejecting `pq_fastscan` and `rabitq` until profile-specific IVF
  payload support lands is the right first-baseline behavior.
- Whether canonical payload-length validation is sufficient here, given the
  `tqvector` type codec rejects noncanonical bits/seed shape before AM
  callbacks receive rows.
- Whether scan-time validation should remain in place to catch old or corrupted
  metadata even though build/insert now reject unsupported storage formats.

## Non-Goals

- Implementing IVF-specific `pq_fastscan` or `rabitq` storage.
- Concurrent insert coverage.
- Planner/admin integration.
- Measurement artifacts.
