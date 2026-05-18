# Task 41 Review Request: Shared Snapshot and Slot Guards

## Scope

This checkpoint promotes two repeated PostgreSQL resource guards into
`src/storage/`:

- `src/storage/snapshot_guard.rs`
- `src/storage/slot_guard.rs`

It then migrates the SPiRE custom scan planner helper from local
`ActiveSnapshotGuard` and `TupleTableSlotGuard` definitions to the shared
types.

Code commit: `e95a128d241eb6c0ebde59721516ad544542459a`

## Safety Invariant

`ActiveSnapshotGuard::latest` registers and pushes a latest snapshot, then
owns the matching active-snapshot pop and snapshot unregister in `Drop`.

`TupleTableSlotGuard::create` owns the tuple slot returned by
`table_slot_create` and drops it through
`ExecDropSingleTupleTableSlot`.

The migrated SPiRE planner helper keeps both guards alive across the index
scan and tuple-slot fetch that need those resources.

## Baseline Impact

Unsafe comment baseline remained stable:

- before: `4239`
- after: `4239`

This is structural consolidation: unsafe operations move from a local
planner helper to shared storage guards. The follow-up value is that the
long-lived scan-state bundles can now be built on shared typed guards rather
than another local mix of raw snapshot and slot pointers.

## Validation

See `artifacts/validation.md`.

Commands run:

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

## Review Focus

- Confirm the shared snapshot guard preserves register/push and
  pop/unregister ordering.
- Confirm the shared tuple-slot guard is appropriate for the
  `table_slot_create` call shape.
- Confirm the SPiRE planner migration keeps snapshot and slot lifetimes wide
  enough for the index scan.
