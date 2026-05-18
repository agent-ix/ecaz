# Artifact Manifest: SPIRE Locked Maintenance Run-Plan SQL Smoke

No measurement artifacts.

- head SHA: `1b4d97c0cf872b29a2d0338ed06e34fd050f03de`
- packet/topic: `30458-spire-locked-maintenance-run-plan-sql-smoke`
- timestamp: `2026-05-05T08:20:26-07:00`
- validation:
  - `cargo pgrx test pg18 test_ec_spire_locked_maintenance_run_plan_no_write_sql`
  - `cargo fmt`
  - `cargo fmt --check`
  - `git diff --check`
- key result lines:
  - `test tests::pg_test_ec_spire_locked_maintenance_run_plan_no_write_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1268 filtered out`
  - `cargo fmt` exited 0 with the repository's stable-rustfmt warnings.
  - `cargo fmt --check` exited 0 with the repository's stable-rustfmt warnings.
  - `git diff --check` exited 0.
