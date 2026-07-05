# Artifact Manifest: SPIRE Maintenance No-Candidate SQL Smoke

No measurement artifacts.

- head SHA: `b17e96483eec088d885e754a7952c510ca4dabc5`
- packet/topic: `30462-spire-maintenance-no-candidate-sql-smoke`
- timestamp: `2026-05-05T08:36:09-07:00`
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
