# Artifact Manifest: SPIRE Maintenance Split Publish Smoke

No measurement artifacts.

- head SHA: `141bb790bd3668cccc53b56a0603df8b637f2aa6`
- packet/topic: `30457-spire-maintenance-split-publish-smoke`
- timestamp: `2026-05-05T08:14:57-07:00`
- validation:
  - `cargo pgrx test pg18 test_ec_spire_maintenance_run_split_publish_sql`
  - `cargo fmt --check`
  - `git diff --check`
- key result lines:
  - `test tests::pg_test_ec_spire_maintenance_run_split_publish_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1267 filtered out`
  - `cargo fmt --check` exited 0 with the repository's stable-rustfmt warnings.
  - `git diff --check` exited 0.
