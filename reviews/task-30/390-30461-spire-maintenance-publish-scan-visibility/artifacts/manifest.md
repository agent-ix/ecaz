# Artifact Manifest: SPIRE Maintenance Publish Scan Visibility

No measurement artifacts.

- head SHA: `bb8a924d69d4fcd03b8b6a93f8132b4182eb7e3d`
- packet/topic: `30461-spire-maintenance-publish-scan-visibility`
- timestamp: `2026-05-05T08:32:17-07:00`
- validation:
  - `cargo pgrx test pg18 maintenance_run`
  - `cargo fmt`
  - `cargo fmt --check`
  - `git diff --check`
- key result lines:
  - `test tests::pg_test_ec_spire_maintenance_run_merge_publish_sql ... ok`
  - `test tests::pg_test_ec_spire_maintenance_run_split_publish_sql ... ok`
  - `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1260 filtered out`
  - `cargo fmt` exited 0 with the repository's stable-rustfmt warnings.
  - `cargo fmt --check` exited 0 with the repository's stable-rustfmt warnings.
  - `git diff --check` exited 0.
