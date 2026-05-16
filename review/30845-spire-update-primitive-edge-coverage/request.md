# SPIRE UPDATE Primitive Edge Coverage

## Scope

This packet tightens focused coverage for
`ec_spire_forward_coordinator_update_tuple_payload(...)` before a transparent
UPDATE front door calls it.

Changes:

- Expands the local `node_id = 0` UPDATE fixture to update two non-embedding
  columns in one helper call.
- Adds a fail-closed fixture proving a missing placement row surfaces
  `ec_spire coordinator update placement row is missing` instead of silently
  no-oping.
- Keeps the existing remote-owned UPDATE forwarding fixture in the same focused
  test filter.
- Updates the Phase 11 tracker with packet `30845`.

## Validation

- `cargo test forward_coordinator_update --lib`
  - result: pass.
  - key lines:
    `test tests::pg_test_ec_spire_forward_coordinator_update_local_sql ... ok`
    `test tests::pg_test_ec_spire_forward_coordinator_update_tuple_payload_sql ... ok`
    `test tests::pg_test_ec_spire_forward_coordinator_update_missing_placement_sql - should panic ... ok`
  - summary: `3 passed; 0 failed; 1645 filtered out`
- `cargo fmt --check`
  - result: pass with the repo's existing stable-rustfmt warnings.
- `git diff --check`
  - result: pass.

## Review Focus

- Confirm the multi-column local fixture covers the same validation path a
  transparent UPDATE front door will use for `node_id = 0`.
- Confirm missing placement should remain a hard error rather than an
  `updated_count = 0` result.
- Confirm no remote UPDATE semantics changed.

## Artifacts

- `review/30845-spire-update-primitive-edge-coverage/artifacts/manifest.md`
- `review/30845-spire-update-primitive-edge-coverage/artifacts/cargo-test-forward-coordinator-update-lib.log`
- `review/30845-spire-update-primitive-edge-coverage/artifacts/cargo-fmt-check.log`
- `review/30845-spire-update-primitive-edge-coverage/artifacts/git-diff-check.log`
