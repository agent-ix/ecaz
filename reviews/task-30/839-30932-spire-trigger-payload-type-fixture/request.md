---
topic: spire-trigger-payload-type-fixture
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30932
stage: phase-12.5
status: open
---

# Review Request: SPIRE Trigger Payload Type Fixture

## Scope

Please review commit `e2d478989647f556bd721a736b0679b0dc31e7e8`
(`Add SPIRE trigger payload type fixture`).

This slice closes the Phase 12.5 trigger payload type fixture row:

- Adds `test_ec_spire_insert_trigger_payload_type_roundtrip_sql`.
- Covers coordinator INSERT trigger forwarding through a loopback remote for:
  - `numeric(12,4)` precision;
  - `timestamptz` value preservation normalized to UTC;
  - nested `json` and `jsonb`;
  - text containing quotes, backslash, and newline;
  - a domain-over-text column;
  - SQL NULL on a nullable column;
  - a NOT NULL violation path that fails closed before a prepared transaction
    is left behind;
  - a default-valued column after PostgreSQL materializes `NEW`.
- Improves coordinator INSERT remote-SQL error formatting so remote
  PostgreSQL message/detail/hint fields are preserved instead of collapsing to
  `db error`.

The schema-drift fingerprint rows remain open; this packet only covers payload
type behavior for the existing JSON bridge.

## Review Focus

- Confirm the fixture covers the listed Phase 12.5 trigger payload classes
  without overclaiming typed transport or schema-drift fingerprint work.
- Confirm preserving remote `postgres::Error` message/detail/hint in the
  coordinator INSERT wrapper is an acceptable operator-facing improvement.
- Confirm the NOT NULL path proves no remote row and no SPIRE prepared xact are
  left after the failed dispatch.

## Validation

Artifacts are packet-local under `artifacts/` and described in
`artifacts/manifest.md`.

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_ec_spire_insert_trigger_payload_type_roundtrip_sql`

Key result: the focused PG18 test passed with `1 passed; 0 failed; 1685 filtered out`.
