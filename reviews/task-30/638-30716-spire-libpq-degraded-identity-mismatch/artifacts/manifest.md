# Artifact Manifest

head_sha: 2610837d3898d9652faf977d8db3a99bcda74cf0
packet: 30716-spire-libpq-degraded-identity-mismatch
lane: Phase 11 Stage C production libpq coordinator
timestamp: 2026-05-10T07:28:02Z

## cargo-check-pg18.log

- Command: `cargo check --no-default-features --features pg18`
- Fixture: PG18 compile check.
- Storage format / rerank mode: not applicable.
- Isolated/shared surface: code compile only.
- Key result:
  - `Finished dev profile ... target(s) in 0.12s`
  - command exit code 0.

## cargo-pgrx-pg18-degraded-identity-mismatch.log

- Command: `cargo pgrx test pg18 test_ec_spire_libpq_degraded_identity_mismatch_skips`
- Fixture: PG18 loopback coordinator/remote pair with a RaBitQ remote-serving
  index and a descriptor identity that does not match the live endpoint
  fingerprint under degraded consistency.
- Storage format / rerank mode: remote loopback index uses
  `storage_format = 'rabitq'`; rerank mode is not a benchmark variable.
- Isolated/shared surface: loopback diagnostic executor surface; no
  performance claim.
- Key result:
  - `test tests::pg_test_ec_spire_libpq_degraded_identity_mismatch_skips ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1522 filtered out`

## cargo-pgrx-pg18-libpq-loopback.log

- Command: `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
- Fixture: PG18 ready loopback remote executor with compact receive, remote
  heap receive, coordinator summary, and identity-cache summary assertions.
- Storage format / rerank mode: remote loopback index uses
  `storage_format = 'rabitq'`; rerank mode is not a benchmark variable.
- Isolated/shared surface: loopback diagnostic executor surface; no
  performance claim.
- Key result:
  - `test tests::pg_test_ec_spire_remote_search_libpq_executor_loopback_empty ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1522 filtered out`

## cargo-pgrx-pg18-strict-identity-mismatch.log

- Command: `cargo pgrx test pg18 test_ec_spire_libpq_rejects_identity_mismatch`
- Fixture: PG18 loopback coordinator/remote pair with a RaBitQ remote-serving
  index and a descriptor identity that does not match the live endpoint
  fingerprint under strict consistency.
- Storage format / rerank mode: remote loopback index uses
  `storage_format = 'rabitq'`; rerank mode is not a benchmark variable.
- Isolated/shared surface: loopback diagnostic executor surface; no
  performance claim.
- Key result:
  - `test tests::pg_test_ec_spire_libpq_rejects_identity_mismatch - should panic ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1522 filtered out`

## git-diff-check.log

- Command: `git diff --check`
- Fixture: whitespace/check-only validation.
- Storage format / rerank mode: not applicable.
- Isolated/shared surface: not applicable.
- Key result:
  - command exit code 0.
