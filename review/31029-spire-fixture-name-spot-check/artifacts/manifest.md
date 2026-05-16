# Artifact Manifest: 31029 SPIRE fixture name spot-check

Head SHA: `5aa4dbdf1d337e5b6621d7e659edaaa1aaaadf15`

Packet/topic: `31029-spire-fixture-name-spot-check`

Timestamp: `2026-05-14T03:03:55Z`

Surface note: this packet records tracker evidence only. No benchmark lane,
fixture corpus, storage format, rerank mode, or isolated/shared index surface
applies.

## Artifacts

### `selected-fixture-names.log`

- Command: `script -q -e -c 'rg -o "test_(pg18_)?ec_spire_[A-Za-z0-9_]+" plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/tasks/task30-phase12-spire-production-hardening.md plan/tasks/task30-phase12a-spire*.md plan/tasks/task30-phase12b-spire-cleanup.md | sed "s/^.*://" | sort -u | shuf -n 10' review/31029-spire-fixture-name-spot-check/artifacts/selected-fixture-names.log`
- Post-processing: removed `script` wrapper lines and CR characters from the
  selection log before using it as input to the location check.
- Key result: ten tracker strings selected.

### `fixture-location-check.log`

- Command: `script -q -e -c 'while IFS= read -r name; do case "$name" in ""|Script\ started*|Script\ done*) continue;; esac; printf "== %s ==\n" "$name"; rg -n --fixed-strings "$name" src/tests; done < review/31029-spire-fixture-name-spot-check/artifacts/selected-fixture-names.log' review/31029-spire-fixture-name-spot-check/artifacts/fixture-location-check.log`
- Key result lines:
  - `src/tests/insert.rs:1767: fn test_ec_spire_schema_drift_fails_before_dispatch_sql()`
  - `src/tests/placement.rs:76: fn test_ec_spire_placement_index_oid_lookup_uses_index_sql()`
  - `src/tests/custom_scan.rs:722: fn test_ec_spire_customscan_does_not_replace_local_only_index_plan()`
  - `src/tests/insert.rs:773: fn test_ec_spire_enable_coordinator_insert_trigger_sql()`
  - `src/tests/remote_search.rs:6161: fn test_ec_spire_prod_transport_local_cancel_remote_cancel()`
  - `src/tests/insert.rs:393: fn test_ec_spire_insert_prepare_local_cancel_rolls_back()`
  - `src/tests/remote_search.rs:6558: fn test_ec_spire_prod_receive_local_cancel_remote_cancel()`
  - `src/tests/dml_frontdoor.rs:1062: fn test_ec_spire_dml_frontdoor_primitive_plan_from_decision()`
  - `test_ec_spire_srcid` resolved to six functions in `src/tests/insert.rs`
  - `src/tests/dml_frontdoor.rs:1672: fn test_ec_spire_update_delete_schema_drift_guard_sql()`

### `git-diff-check.log`

- Command: `script -q -e -c 'git diff --check -- plan/tasks/task30-phase12b-spire-cleanup.md' review/31029-spire-fixture-name-spot-check/artifacts/git-diff-check.log`
- Key result: command exited `0`.

### `diff-stat.log`

- Command: `script -q -e -c 'git diff --stat -- plan/tasks/task30-phase12b-spire-cleanup.md' review/31029-spire-fixture-name-spot-check/artifacts/diff-stat.log`
- Key result: `1 file changed, 3 insertions(+), 1 deletion(-)`.
