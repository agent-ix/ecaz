# Task 236 packet 003 artifact manifest

- Head SHA: `45f895d68fb208f6b3cfe4d8b0fdd17d04c6b3db`
- Task / packet: `reviews/task-236/003-pg18-tls-secret-matrix/`
- Timestamp: 2026-08-24 15:53–15:55 America/Los_Angeles
- Lane: local PG18 three-owner DistANN TLS/mTLS and secret-rotation matrix
- Fixture: synthetic 2,000-row, 32-dimensional physical generation; three
  owners; graph degree 32; one distributed index with one owner-local relation
  set per node; no shared-table benchmark surface
- PostgreSQL: isolated PostgreSQL 18.3 build configured with OpenSSL under
  `/home/peter/.ecaz/toolchains/pg18-ssl`
- Extension: debug `pg18,pg_test` build required for diagnostic probes; all
  three nodes unanimously reported the head SHA above
- Run directory: `/home/peter/.ecaz/clusters/task236-tls-secret-matrix`, outside
  the repository because this fixture needs a private OpenSSL-enabled PG18
  build and generated certificate material; the stopped cluster and all keys
  were removed after the cited artifacts were captured
- Storage / rerank: physical DistANN owner-local graph and row tiers; no rerank
  variant axis

## Suite driver

### `task236-tls-security-suite.json`

Checked-in `ecaz bench suite` configuration. The final expansion uses three
owners, `--secure-remote-transport`, and `--tls-security-matrix`; the run
directory is outside the repository and durable output is packet-local.

### `suite-dry-run-manifest-final-pass.json` and `suite-dry-run-final-pass.log`

- Command: `/home/peter/.cargo-target/debug/ecaz bench suite run --config
  reviews/task-236/003-pg18-tls-secret-matrix/artifacts/task236-tls-security-suite.json
  --dry-run --manifest-output
  reviews/task-236/003-pg18-tls-secret-matrix/artifacts/suite-dry-run-manifest-final-pass.json
  --results-output
  reviews/task-236/003-pg18-tls-secret-matrix/artifacts/results-dry-run-final-pass.jsonl
  --log-file
  reviews/task-236/003-pg18-tls-secret-matrix/artifacts/suite-dry-run-final-pass.log`
- Result: expansion includes the secure transport and TLS security matrix flags
  on three owners at the packet-local artifact paths.

### `suite-manifest-final-pass.json`, `results-final-pass.jsonl`, and
`suite-run-final-pass.log`

- Command: `/home/peter/.cargo-target/debug/ecaz bench suite run --config
  reviews/task-236/003-pg18-tls-secret-matrix/artifacts/task236-tls-security-suite.json
  --manifest-output
  reviews/task-236/003-pg18-tls-secret-matrix/artifacts/suite-manifest-final-pass.json
  --results-output
  reviews/task-236/003-pg18-tls-secret-matrix/artifacts/results-final-pass.jsonl
  --log-file
  reviews/task-236/003-pg18-tls-secret-matrix/artifacts/suite-run-final-pass.log`
- Result: suite step `succeeded`, exit 0, 14,008 ms, with 22 normalized result
  rows. The manifest and extension preflight both report the head SHA above.
- Key results: three owners reached `Published`; serving returned 10/10 rows;
  a routed insert committed on remote owner 2; the matrix passed 13 cells.

## Live matrix evidence

### `run-final-pass/fixture/task236-tls-security-matrix.log`

- Valid verify-full mutual TLS connected with TLS 1.3.
- Wrong CA, wrong hostname, missing/incorrect client certificate,
  expired/not-yet-valid client certificate, and plaintext-to-the-TLS-only role
  all failed closed as `secure_connect_failed`.
- Unsupported `sslmode=prefer` failed as `tls_option_unsupported`.
- An exact-peer socket reset during connection establishment failed closed;
  the disarmed recovery connected with TLS 1.3.
- Credential rotation replaced rather than grew the pool
  (`pool_before=2 pool_after=2 pool_reused=2`), with the observed rotated
  handshake at 466 ms and pooled reuse at 3 ms.
- Live node-descriptor, serialized-generation, EXPLAIN, and topology-status
  inspection reported zero secret exposures.

### `run-final-pass/fixture.log`

Full compact fixture output, including extension provenance, ready/published
topology, serving, remote-owner DML, the 13 matrix results, and normal shutdown.

### `run-final-pass/fixture/node{1,2,3}-postgres.log`

Packet-local PG18 server logs inspected for raw conninfo, certificate/key path,
private-key content, and secret-reference exposure.

### `run-final-pass/fixture/distann-remote-socket-fault.marker`

Exact-peer provider evidence for the armed socket-reset handshake cell.

### `secret-exposure-scan.log`

Accepted-artifact scan: zero matches for raw conninfo/TLS path/private-key and
secret-reference patterns.

## Static validation

### `cargo-check-ecaz-cli.log`

- Command: `cargo check -p ecaz-cli`
- Result: exit 0. The only warning is the pre-existing unused `path` field in
  corpus loading.

### `suite-contract-test.log`

- Command: `cargo test -p ecaz-cli
  distann_local_multinode_expands_secure_remote_transport`
- Result: 1 passed, 0 failed; proves suite validation, flag expansion, and the
  packet artifact contract.
