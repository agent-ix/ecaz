# Task 39 / 045 — Extract `health_snapshot_from_diagnostics`

## Goal

Move `health_snapshot_from_diagnostics` from
`coordinator/diagnostics.rs` into the sibling helpers file at 100%
coverage. The function is the largest single pure helper still inside
the production diagnostics body (~67 lines, 7 branches).

## Code Change

`src/am/ec_spire/coordinator/diagnostics_helpers.rs`:

- Adds `health_snapshot_from_diagnostics(&SpireActiveSnapshotDiagnostics)
  -> SpireIndexHealthSnapshot`. Body byte-for-byte identical to the
  removed version.

`src/am/ec_spire/coordinator/diagnostics.rs`:

- Removes the moved definition.

`hardening/careful/src/spire_diagnostics_helpers.rs`:

- Shims `SpireActiveSnapshotDiagnostics` (22 fields) and
  `SpireIndexHealthSnapshot` (14 fields) — both copied verbatim from
  `coordinator/types.rs`.
- New test `miri_health_snapshot_walks_every_status_branch` covers all
  7 status branches (empty, unavailable_placements, stale_placements,
  skipped_placements, maintenance_recommended, degraded_consistency,
  ok) plus the priority ordering between unavailable and stale.

## Baseline Net Effect

| File | Pre-packet (044) | This packet |
| --- | ---: | ---: |
| `am/ec_spire/coordinator/diagnostics_helpers.rs` | 100.00 (220 lines) | **100.00 (284 lines)** |
| `am/ec_spire/coordinator/diagnostics.rs` | 0.00 (549 lines) | 0.00 (485 lines) |

Net: ~64 more production lines now exercise under `make coverage`
(this is on top of the 220 already extracted by packets 040 and 044).

## Validation

Artifacts under
`reviews/task-39/045-health-snapshot-extract/artifacts/`:

- `health-snapshot-extract-focused-tests.log`: **529 passed**, 0 failed
  (was 528 after packet 044; +1 new test).
- `coverage/summary.txt` + JSON.
- `coverage-delta-check.log`: every baseline row green.
- `coverage-baseline-check.log`: **42 critical paths complete**.
- Production `cargo check --features pg18 --no-default-features`
  clean (function body unchanged).
