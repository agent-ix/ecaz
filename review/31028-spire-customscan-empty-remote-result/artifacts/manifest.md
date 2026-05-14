# Artifact Manifest: 31028 SPIRE CustomScan empty remote result

Head SHA: `6fbdec7d1612b63786c497aeaf5dae07539187f7`

Packet/topic: `31028-spire-customscan-empty-remote-result`

Timestamp: `2026-05-14T02:59:51Z`

Lane / fixture / storage format / rerank mode: focused PG18 pgrx fixture,
loopback remote CustomScan DML PK-select empty result, default ec_spire storage
for the DML fixture, no rerank lane.

Surface note: this packet adds a Rust pg_test fixture. The run uses the normal
PG18 pgrx test database, not an isolated benchmark corpus. The fixture creates
one coordinator index and one loopback remote index inside the test.

## Artifacts

### Final Static Checks

#### `cargo-fmt-check-plan.log`

- Command: `script -q -e -c 'cargo fmt --check' review/31028-spire-customscan-empty-remote-result/artifacts/cargo-fmt-check-plan.log`
- Key result: command exited `0`.

#### `git-diff-check-plan.log`

- Command: `script -q -e -c 'git diff --check -- plan/tasks/task30-phase12b-spire-cleanup.md src/tests/custom_scan.rs' review/31028-spire-customscan-empty-remote-result/artifacts/git-diff-check-plan.log`
- Key result: command exited `0`.

#### `location-check-final.log`

- Command: `script -q -e -c 'rg -n "test_ec_spire_customscan_empty_remote_result_returns_no_rows|Empty-remote-result CustomScan fixture|remote_select_status|tuple_transport_status|not_applicable" plan/tasks/task30-phase12b-spire-cleanup.md src/tests/custom_scan.rs' review/31028-spire-customscan-empty-remote-result/artifacts/location-check-final.log`
- Key result lines:
  - `src/tests/custom_scan.rs:288: fn test_ec_spire_customscan_empty_remote_result_returns_no_rows()`
  - `src/tests/custom_scan.rs:366: let remote_select_status = Spi::get_one::<String>(`
  - `src/tests/custom_scan.rs:445: json_plan.contains("\"tuple_transport_status\": \"ready\"")`
  - `src/tests/custom_scan.rs:449: !json_plan.contains("not_applicable")`
  - `plan/tasks/task30-phase12b-spire-cleanup.md:327:- [x] Empty-remote-result CustomScan fixture`

#### `diff-stat-final.log`

- Command: `script -q -e -c 'git diff --stat -- plan/tasks/task30-phase12b-spire-cleanup.md src/tests/custom_scan.rs' review/31028-spire-customscan-empty-remote-result/artifacts/diff-stat-final.log`
- Key result: `2 files changed, 173 insertions(+), 2 deletions(-)`.

### Final PG18 Check

#### `pg18-test-customscan-empty-remote-result-final.log`

- Command: `script -q -e -c 'cargo pgrx test pg18 test_ec_spire_customscan_empty_remote_result_returns_no_rows' review/31028-spire-customscan-empty-remote-result/artifacts/pg18-test-customscan-empty-remote-result-final.log`
- Key result: `1 passed; 0 failed; 0 ignored; 0 measured; 1712 filtered out; finished in 35.05s`

## Failed Intermediate Runs

These are included to explain fixture-shape corrections, not as accepted
validation.

### `pg18-test-customscan-empty-remote-result.log`

- Initial vector/delete-delta setup failed before behavior validation because
  the broad `DELETE FROM ...` statement hit the distributed DML shape guard.
- Key result: `0 passed; 1 failed`.

### `pg18-test-customscan-empty-remote-result-rerun.log`

- Primary-key equality deletes were accepted, but the remote tuple-payload
  endpoint still returned one row after delete deltas.
- Key result: `left: 1`, `right: 0`.

### `pg18-test-customscan-empty-remote-result-dml.log`

- First DML PK-select version correctly returned `selected_count = 0`, but the
  expected status string was too specific.
- Key result: left was `true|0|remote_select_ready|done`; expected
  `true|0|remote_select_empty|done`.

### `pg18-test-customscan-empty-remote-result-status.log`

- Status expectation was fixed, but text EXPLAIN used the stable callback line
  `node: EcSpireDistributedScan` rather than the scan title string.
- Key result: text plan included `node: EcSpireDistributedScan` and
  `tuple_transport_status: ready`.
