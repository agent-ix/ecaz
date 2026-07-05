# Review Request: Task 28 IVF Admin Snapshot

Status: open
Owner: coder2
Code checkpoint: 3f4f58a7076af65b0a661597fa351468c1072e8e
Branch: task28-ivf
Date: 2026-04-25

## Scope

This packet covers the Phase 7 admin-snapshot checkpoint for the first
`ec_ivf` access method baseline.

Changes in scope:

- Add `ec_ivf_index_admin_snapshot(regclass)` as a SQL diagnostics surface.
- Report metadata shape, persisted and effective `nprobe`, session override
  state, storage/rerank profiles, live/dead/drift counters, list distribution,
  REINDEX recommendation state, and planner inputs (`index_pages`, `reltuples`).
- Move IVF effective `nprobe` resolution into a reusable options helper shared
  by scan and admin code.
- Mark the Phase 7 admin-snapshot checklist item complete in
  `plan/tasks/28-ivf-access-method.md`.

## Files

- `src/am/ec_ivf/admin.rs`
- `src/am/ec_ivf/options.rs`
- `src/am/ec_ivf/scan.rs`
- `src/am/ec_ivf/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

## Validation

PG18-only validation:

- `cargo check --no-default-features --features pg18 --tests`
- `cargo pgrx test pg18 test_ec_ivf_admin_snapshot`
- `git diff --check`

PostgreSQL version: 18.3 via pgrx `pg18`.

No measurement claims are made in this packet.

## Review Focus

- Whether the admin snapshot exposes the right planner inputs before the cost
  model lands.
- Whether centralizing effective `nprobe` resolution in `options.rs` is the
  right shared location for scan/admin/planner code.
- Whether the snapshot has too many drift fields duplicated from
  `ec_ivf_index_drift_snapshot`, or whether that duplication is useful for a
  single operator-facing admin surface.

## Non-Goals

- Activating finite IVF planner costs.
- Adding EXPLAIN counters.
- Adding PG18 ReadStream or shared stats wiring.
- Measurement artifacts.
