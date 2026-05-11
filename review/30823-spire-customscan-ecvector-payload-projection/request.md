# Review Request: SPIRE CustomScan Ecvector Payload Projection

## Scope

Feedback follow-up for the P2 in
`review/30816-spire-customscan-payload-scalar-gate/feedback/2026-05-11-001-reviewer.md`.

The reviewer asked to confirm that projected `ecvector` columns do not trip the
temporary tuple-payload array/composite gate. `ecvector` is declared as a
PostgreSQL base type in the extension SQL, but this packet pins the behavior in
the CustomScan path itself.

This slice:

- Extends the loopback CustomScan fixture to project remote `embedding` through
  `ecvector_to_real_array(embedding, 4, false)`.
- Asserts the returned vector is `[1.0, 0.0]`, proving the remote `ecvector`
  payload survived validation, JSON bridge transport, type input, and SQL
  expression evaluation.
- Adds a short comment to the validator clarifying that the v1 gate rejects
  PostgreSQL arrays and row composites while allowing scalar base/domain types
  such as `ecvector`, json/jsonb, enum, and range through type input.

This does not remove the JSON tuple-payload bridge or broaden support to array
or composite payload columns.

## Validation

- `cargo test customscan_returns_loopback_remote_tuple_payload --lib`
  - Passed: 1 test.
- `cargo fmt --check`
  - Passed with the repository's existing stable-rustfmt warnings about
    nightly-only import options.
- `git diff --check`
  - Passed.
- `git diff --cached --check`
  - Passed before the code commit.

## Review Focus

- Confirm that projecting `embedding` through `ecvector_to_real_array(...)` is
  sufficient proof that `ecvector` is not rejected as an array-like type by the
  CustomScan payload validator.
- Confirm the validator comment accurately describes the interim scalar gate.

## Artifacts

- `review/30823-spire-customscan-ecvector-payload-projection/artifacts/manifest.md`
- `review/30823-spire-customscan-ecvector-payload-projection/artifacts/cargo-test-customscan-ecvector-projection-lib.log`
- `review/30823-spire-customscan-ecvector-payload-projection/artifacts/cargo-fmt-check.log`
- `review/30823-spire-customscan-ecvector-payload-projection/artifacts/git-diff-check.log`
- `review/30823-spire-customscan-ecvector-payload-projection/artifacts/git-diff-cached-check.log`
