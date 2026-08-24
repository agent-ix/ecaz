# Task 236: ec_distann Secure Transport and Secret Resolution

Status: **shared secure connector implementation complete; packet 002
review-open; PG18 TLS/mTLS/rotation matrix and security closeout pending**
(2026-08-24; packet:
`reviews/task-236/002-shared-secure-connector/`). Priority: P0 transport
security before Task 228.

## Why

The production ec_distann async transport still calls
`tokio_postgres::Config::connect(NoTls)`. FR-079 records this as the Task-214
F10 implementation gap. Distributed generation catalogs correctly persist a
`conninfo_secret_name`, and executor paths resolve it to in-memory conninfo,
but the connection layer does not preserve the resolved libpq TLS contract.

SPIRE already contains a rustls-backed tokio-postgres connector and conninfo
normalization/security behavior. ec_distann should reuse or extract that
reviewed substrate rather than grow a second incompatible TLS parser.

## Goal

Replace hardwired `NoTls` on every production ec_distann coordinator-to-owner
path with one secret-backed connector that preserves supported `sslmode`, CA,
client-certificate, private-key, hostname-verification, and channel-binding
semantics while keeping raw conninfo and secret material out of persisted
state, SQL diagnostics, logs, errors, and review artifacts.

## Entry conditions

1. The plan packet inventories async read, build/handoff/lifecycle, DML,
   callback/reaper, and traversal-replica connections, including any sync
   `postgres` clients outside `remote_transport.rs`.
2. The SPIRE production TLS connector and secret-resolution behavior are
   reviewed for extraction/reuse; intentional ec_distann differences are
   documented before implementation.
3. Supported and rejected conninfo/TLS options are frozen. Unsupported modes
   fail closed with a stable category rather than silently downgrading.

## Required implementation

### P1 — Shared secure connector

- Extract or directly reuse one connector implementation for SPIRE and
  ec_distann where practical. Do not duplicate certificate verification or
  conninfo parsing rules.
- Resolve `conninfo_secret_name` only inside executor/connection setup and keep
  the raw value in bounded in-memory connection state only.
- Preserve supported `sslmode`, CA roots, client certificate/key, hostname
  verification, and channel binding from the resolved secret. Never fall back
  from a requested verified mode to plaintext or unverified TLS.
- Key pooled connections by the stable endpoint/secret identity plus the TLS
  configuration identity needed to prevent reuse across rotated credentials or
  changed verification policy. Define bounded invalidation on rotation.
- Keep explicitly configured local loopback plaintext available only where the
  deployment/test policy permits it; production remote defaults must not depend
  on implicit `NoTls`.

### P2 — Sanitized failures and secret hygiene

- Map missing secret, malformed conninfo, unsupported TLS option, DNS/connect,
  CA/certificate, hostname, client-auth, and channel-binding failures to stable
  sanitized categories and operator hints.
- Prove that raw conninfo, secret names where prohibited, certificate/key
  contents, row payload, and unsanitized server errors do not appear in SQL
  results, NOTICE/ERROR text, EXPLAIN, logs, status surfaces, manifests, or
  benchmark artifacts.
- Preserve endpoint identity, logical-index identity, schema/fingerprint, and
  owner validation after the secure connection is established.

### P3 — PG18 multinode security matrix

- Cover valid verify-full, wrong CA, wrong hostname, expired/not-yet-valid
  certificate, missing/incorrect client certificate, rotated secret,
  plaintext-disabled remote, unsupported sslmode, and connection reset during
  handshake.
- Exercise read, materialization, build/handoff/publish, DML, and recovery
  entrypoints through the secure connector.
- Inspect internal catalogs, errors, logs, EXPLAIN/status output, and packet
  artifacts for secret exposure.
- Measure handshake and pooled-reuse overhead so Task 228's later network
  attribution uses the production security substrate rather than `NoTls`.

## Non-goals

- Designing a non-PostgreSQL RPC protocol.
- Persisting raw conninfo or certificate/key bytes in generation artifacts.
- Certificate issuance, fleet-wide PKI automation, or secret-provider
  deployment outside the existing provider contract.
- Weakening verification to make a fixture pass.

## Acceptance

1. Structural inspection finds no hardwired `NoTls` on a production
   ec_distann remote path; any fixture-only exception is explicit and gated.
2. Verified TLS and mutual-auth cells succeed, negative certificate/identity
   cells fail closed, and rotation invalidates affected pooled sessions.
3. Secret-exposure inspection reports zero raw conninfo/key/payload leaks.
4. Outside security review accepts NFR-014 and FR-079 transport incorporation.

## Required review packets

1. `reviews/task-236/001-plan-and-connection-inventory/`
2. `reviews/task-236/002-shared-secure-connector/`
3. `reviews/task-236/003-pg18-tls-secret-matrix/`
4. `reviews/task-236/004-security-closeout/`

## References

- FR-078, FR-079 implementation gap F10, FR-081, FR-082, FR-083
- NFR-014
- Tasks 214, 228, 234, and 235
- `src/am/ec_spire/coordinator/remote_candidates/tls.rs`
- `src/am/ec_distann/node_registry.rs`
- `src/am/ec_distann/remote_transport.rs`
