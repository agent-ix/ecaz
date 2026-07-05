# Artifact Manifest: SPIRE Maintenance Merge Publish Smoke

No measurement artifacts.

- head SHA: `ca28659e0a2d84a8f7f9e01b4b58994ab5664faa`
- packet/topic: `30456-spire-maintenance-merge-publish-smoke`
- timestamp: `2026-05-05T08:10:45-07:00`
- validation:
  - `cargo test selected_scheduled_replacement_leaf_rows_keeps_empty_affected_leaf --lib`
  - `cargo pgrx test pg18 test_ec_spire_maintenance_run_merge_publish_sql`
  - `cargo fmt --check`
  - `git diff --check`
- key result lines:
  - `test am::ec_spire::update::tests::selected_scheduled_replacement_leaf_rows_keeps_empty_affected_leaf ... ok`
  - `test tests::pg_test_ec_spire_maintenance_run_merge_publish_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1266 filtered out`
  - `cargo fmt --check` exited 0 with the repository's stable-rustfmt warnings.
  - `git diff --check` exited 0.
