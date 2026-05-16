# SPIRE PK SELECT Duplicate Guard

## Scope

This packet addresses reviewer feedback from `30840` by making the PK SELECT
primitive fail closed if a lookup returns more than one tuple.

Changes:

- Adds `selected_count > 1` guards to the coordinator-local branch,
  coordinator-remote branch, and direct remote endpoint.
- Adds PG18 coverage with a local duplicate-key table proving
  `ec_spire_forward_coordinator_select_tuple_payload(...)` errors instead of
  returning an ambiguous tuple payload.
- Documents PK-read idempotent retry behavior and duplicate-match rejection in
  ADR-069.
- Updates the Phase 11 tracker with packet `30846`.

## Validation

- `cargo test forward_coordinator_select --lib`
  - result: pass.
  - key lines:
    `test tests::pg_test_ec_spire_forward_coordinator_select_local_sql ... ok`
    `test tests::pg_test_ec_spire_forward_coordinator_select_tuple_payload_sql ... ok`
    `test tests::pg_test_ec_spire_forward_coordinator_select_rejects_multirow_sql - should panic ... ok`
  - summary: `3 passed; 0 failed; 1646 filtered out`
- `cargo fmt --check`
  - result: pass with the repo's existing stable-rustfmt warnings.
- `git diff --check`
  - result: pass.

## Review Focus

- Confirm the coordinator and remote `selected_count > 1` guards are the right
  fail-closed policy for schema drift.
- Confirm `selected_count = 0` remains a valid "not found" result.
- Confirm ADR-069's PK-read retry/idempotency note is sufficient.

## Artifacts

- `review/30846-spire-pk-select-duplicate-guard/artifacts/manifest.md`
- `review/30846-spire-pk-select-duplicate-guard/artifacts/cargo-test-forward-coordinator-select-lib.log`
- `review/30846-spire-pk-select-duplicate-guard/artifacts/cargo-fmt-check.log`
- `review/30846-spire-pk-select-duplicate-guard/artifacts/git-diff-check.log`
