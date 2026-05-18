# Review Request: Task 28 IVF PG18 Stats Counters

Status: open
Owner: coder2
Date: 2026-04-25
Branch: `task28-ivf`
Code checkpoint: `27af3ec29c660d71c566607fe37de1716e3c2bef`

## Scope

- Wire `ec_ivf` scans into the existing PG18-compatible `ecaz_stats()`
  counter surface.
- Count IVF scan starts, centroid/posting distance calculations, and
  posting-list pages read.
- Flush per-scan deltas through the existing shared pgstat shim path, with the
  existing backend-local fallback when the custom pgstat kind is not
  preload-active.
- Add PG18 SQL coverage that checks `ecaz_stats()` counter deltas around an
  `ec_ivf` index scan.
- Record the Phase 7 stats checkpoint in `plan/tasks/28-ivf-access-method.md`.

## Files

- `src/am/ec_ivf/scan.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

## Validation

- `cargo check --no-default-features --features pg18 --tests`
- `cargo pgrx test pg18 test_pg18_ecaz_stats_reports_backend_local_counters_for_ec_ivf`
- `git diff --check`

No PG17 tests were run for this checkpoint.

## Review Focus

- Whether mapping IVF posting-list pages onto `total_linear_pages` is the right
  fit for the existing shared stats vocabulary.
- Whether `total_distance_calcs` should count one event per centroid/posting
  payload score, rather than one event per heap TID candidate.
- Whether flushing stats deltas on `amrescan` and `amendscan` matches the
  existing `ec_hnsw` lifecycle closely enough.

## Non-Goals

- New IVF-specific SQL stats columns.
- PG18 ReadStream wiring.
- Shared-preload pgstat validation; this uses the existing backend-local
  fallback path in the ordinary `cargo pgrx test pg18` lane.
