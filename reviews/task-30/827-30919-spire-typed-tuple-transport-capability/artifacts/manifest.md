---
topic: spire-typed-tuple-transport-capability
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30919
---

# Artifact Manifest

Head SHA: `119fd741a20642b289f01a24c2eb84b271b56ed1`

Packet/topic: `30919-spire-typed-tuple-transport-capability`

Timestamp: `2026-05-12T12:07:26-07:00`

Surface: local PG18 pgrx endpoint identity and contract tests.

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

### `cargo-pgrx-test-endpoint-identity.log`

- Command: `cargo pgrx test pg18 test_ec_spire_remote_search_endpoint`
- Exit code: 0
- Lane / fixture: PG18 remote search endpoint identity fixture.
- Storage format / rerank mode: `storage_format = 'rabitq'`, no rerank mode
  override.
- Shared-table vs isolated: isolated endpoint identity test table/index.
- Key result lines:
  - `test tests::pg_test_ec_spire_remote_search_endpoint_identity ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1679 filtered out`

### `cargo-pgrx-test-receive-contract.log`

- Command: `cargo pgrx test pg18 test_ec_spire_remote_search_receive_contract`
- Exit code: 0
- Lane / fixture: PG18 remote search receive and endpoint contract fixture.
- Storage format / rerank mode: contract-only SQL surface, no storage format
  or rerank mode.
- Shared-table vs isolated: no table/index fixture; static contract rows.
- Key result lines:
  - `test tests::pg_test_ec_spire_remote_search_receive_contract ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1679 filtered out`
