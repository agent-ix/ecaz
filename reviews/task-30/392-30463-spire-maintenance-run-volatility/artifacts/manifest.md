# Artifact Manifest: SPIRE Maintenance Run Volatility

No measurement artifacts.

- head SHA: `f3dd928729e269cd8553f78a1941fd3cf4e75931`
- packet/topic: `30463-spire-maintenance-run-volatility`
- timestamp: `2026-05-05T08:40:46-07:00`
- validation:
  - `cargo pgrx test pg18 test_ec_spire_maintenance_run_no_candidate_sql`
  - `cargo fmt`
  - `cargo fmt --check`
  - `git diff --check`
- key result lines:
  - `test tests::pg_test_ec_spire_maintenance_run_no_candidate_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1269 filtered out`
  - `cargo fmt` exited 0 with the repository's stable-rustfmt warnings.
  - `cargo fmt --check` exited 0 with the repository's stable-rustfmt warnings.
  - `git diff --check` exited 0.
