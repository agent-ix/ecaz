# Artifact Manifest: 31025 SPIRE Cost/Planner Fixture Split

Head SHA: `4eba0c3b1ba2c819cca5264f007d19035fe0db6a`

Packet/topic: `31025-spire-cost-planner-fixture-split`

Timestamp: `2026-05-13T19:06:15-07:00`

Lane / fixture / storage format / rerank mode: Phase 12b cleanup fixture
relocation; SPIRE cost/planner registration fixtures; PostgreSQL/pgrx test
storage; rerank mode not applicable.

Isolation surface: local PG18 pgrx test database; not a measurement run; not
an isolated one-index-per-table or shared-table benchmark surface.

## Artifacts

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Result: pass
- Key lines: rustfmt emitted the repository's stable-channel warnings for
  unstable `imports_granularity` and `group_imports`; command exited 0.

### `git-diff-check.log`

- Command: `git diff --check`
- Result: pass
- Key lines: no output; command exited 0.

### `location-check.log`

- Command: `rg -n 'test_ec_spire_access_method_is_registered|test_ec_spire_operator_classes_are_registered|test_ec_spire_custom_scan_status_registered_fail_closed|include!\(\"cost_and_planner.rs\"\)' src/tests/mod.rs src/tests/cost_and_planner.rs`
- Result: pass
- Key lines:
  - `src/tests/cost_and_planner.rs:2:    fn test_ec_spire_access_method_is_registered()`
  - `src/tests/cost_and_planner.rs:11:    fn test_ec_spire_operator_classes_are_registered()`
  - `src/tests/cost_and_planner.rs:24:    fn test_ec_spire_custom_scan_status_registered_fail_closed()`
  - `src/tests/mod.rs:2492:    include!("cost_and_planner.rs");`

### `line-counts.log`

- Command: `wc -l src/tests/mod.rs src/tests/cost_and_planner.rs src/tests/remote_search.rs`
- Result: informational
- Key lines:
  - `34208 src/tests/mod.rs`
  - `54 src/tests/cost_and_planner.rs`
  - `2634 src/tests/remote_search.rs`

### `pg18-test-access-method-registered.log`

- Command: `cargo pgrx test pg18 test_ec_spire_access_method_is_registered`
- Result: pass
- Key line: `test tests::pg_test_ec_spire_access_method_is_registered ... ok`

### `pg18-test-operator-classes-registered.log`

- Command: `cargo pgrx test pg18 test_ec_spire_operator_classes_are_registered`
- Result: pass
- Key line: `test tests::pg_test_ec_spire_operator_classes_are_registered ... ok`

### `pg18-test-custom-scan-status-registered.log`

- Command: `cargo pgrx test pg18 test_ec_spire_custom_scan_status_registered_fail_closed`
- Result: pass
- Key line: `test tests::pg_test_ec_spire_custom_scan_status_registered_fail_closed ... ok`
