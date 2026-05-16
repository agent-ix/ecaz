# Review Request: SPIRE Placement Batch Hardening

## Scope

Feedback follow-up for the P2/P3 items in
`review/30819-spire-placement-batch-registration/feedback/2026-05-11-001-reviewer.md`.

The reviewer accepted the batch-registration primitive but asked for clearer
behavior before coordinator-routed writes and bulk-load tooling consume it. This
packet tightens malformed-entry handling and pins the v1 transaction/type-shape
contract.

This slice:

- Converts `ec_spire_register_placement_batch(...)` from a SQL wrapper to a
  PL/pgSQL function in bootstrap and upgrade SQL.
- Explicitly rejects NULL array elements with
  `ec_spire_register_placement_batch entries[N] is NULL`.
- Preserves strict catalog enforcement for duplicate primary keys and invalid
  placement-entry fields.
- Adds focused PG18 tests for empty batches, NULL entries, duplicate keys, and
  invalid source identities.
- Documents that the function runs inside the caller transaction and that the
  v1 `ec_spire_placement_entry` field order is frozen.
- Updates the Phase 11 tracker to record the hardening packet.

This does not add coordinator-routed INSERT, 2PC, or bulk-load tooling. It only
hardens the placement-registration primitive those paths will depend on.

## Validation

- `cargo test placement_batch --lib`
  - Passed: 4 tests.
- `cargo fmt --check`
  - Passed with the repository's existing stable-rustfmt warnings about
    nightly-only import options.
- `git diff --check`
  - Passed.
- `git diff --cached --check`
  - Passed before the code commit.

## Review Focus

- Confirm the explicit NULL-entry error is the right contract for malformed
  `ec_spire_placement_entry[]` inputs.
- Confirm preserving primary-key and catalog constraint violations is preferable
  to wrapping every malformed field with custom errors.
- Confirm the ADR text correctly pins all-or-nothing caller-transaction
  behavior and the v1 composite field order before downstream write paths rely
  on this function.

## Artifacts

- `review/30825-spire-placement-batch-hardening/artifacts/manifest.md`
- `review/30825-spire-placement-batch-hardening/artifacts/cargo-test-placement-batch-lib.log`
- `review/30825-spire-placement-batch-hardening/artifacts/cargo-fmt-check.log`
- `review/30825-spire-placement-batch-hardening/artifacts/git-diff-check.log`
- `review/30825-spire-placement-batch-hardening/artifacts/git-diff-cached-check.log`
