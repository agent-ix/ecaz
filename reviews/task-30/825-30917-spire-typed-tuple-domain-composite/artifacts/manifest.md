---
topic: spire-typed-tuple-domain-composite
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30917
---

# Artifact Manifest

Head SHA: `cb40c9d30ef500f3c7d810bb66366182b5322a3b`

Packet/topic: `30917-spire-typed-tuple-domain-composite`

Timestamp: `2026-05-12T11:36:07-07:00`

Surface: local PG18 pgrx test surface, isolated one-index-per-table fixtures.

## Artifacts

### `git-diff-check.log`

- Command: `git diff --check HEAD^ HEAD`
- Exit code: 0
- Key result: no whitespace errors.

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Exit code: 0
- Key result: formatting check passed. The log contains the existing stable
  toolchain warnings for unstable rustfmt import-group options.

### `cargo-pgrx-test-typed-domain-composite.log`

- Command:
  `cargo pgrx test pg18 test_ec_spire_typed_tuple_payload_domain_composite_sql`
- Exit code: 0
- Lane / fixture: PG18 typed tuple payload domain/composite fixture.
- Storage format / rerank mode: normal `ec_spire` test index, no rerank mode
  override.
- Shared-table vs isolated: isolated table/index
  `ec_spire_tuple_payload_typed_record_sql` /
  `ec_spire_tuple_payload_typed_record_idx`.
- Key result lines:
  - `test tests::pg_test_ec_spire_typed_tuple_payload_domain_composite_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1679 filtered out`

### `cargo-pgrx-test-typed-tuple-payload.log`

- Command: `cargo pgrx test pg18 test_ec_spire_typed_tuple_payload`
- Exit code: 0
- Lane / fixture: PG18 typed tuple payload regression filter.
- Storage format / rerank mode: normal `ec_spire` test indexes, no rerank mode
  override.
- Shared-table vs isolated: isolated one-index-per-table fixtures for scalar,
  NULL/array, and domain/composite typed payloads.
- Key result lines:
  - `test tests::pg_test_ec_spire_typed_tuple_payload_domain_composite_sql ... ok`
  - `test tests::pg_test_ec_spire_typed_tuple_payload_null_array_sql ... ok`
  - `test tests::pg_test_ec_spire_typed_tuple_payload_scalar_parity_sql ... ok`
  - `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1677 filtered out`
