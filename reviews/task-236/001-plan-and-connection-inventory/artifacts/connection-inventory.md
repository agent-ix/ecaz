# Task 236 connection and security-contract inventory

Inventory head: `308eea630beb0f619bdba71c62686c5ca4140174`

This inventory covers extension production code under `src/am/ec_distann`.
The `ecaz dev distann-multicluster` CLI uses `NoTls` to operate its local test
clusters; those fixture-driver connections are not coordinator-to-owner
production transport and are outside the connector replacement.

## Connection sinks

| Sink | Callers / work classes | Current transport | Task 236 disposition |
|---|---|---|---|
| `remote_transport::open_remote_connection` | pooled physical head, crown, gateway, expand, materialize, row-payload materialize, traversal-replica stream; build begin/stage/seal/abort/publish/retire/reclaim; DML insert/backlink/tombstone | async `tokio_postgres::Config::connect(NoTls)` | replace with shared parsed secure connector; retain the matching TLS config on each pooled client so cancellation uses the same policy |
| `remote_transport::await_remote` / `await_remote_read` | cancellation for every pooled async work class above | `CancelToken::cancel_query(NoTls)` | use the pooled connection's connector identity/config; never deliver a cancel over a weaker transport |
| `spire_remote_search_libpq_connect_with_session_timeouts` called from `remote_transport.rs` | insert intent creation/state transitions, pre-commit decision, commit/rollback callback, prepared-xact reaper | sync shared SPIRE connector | move behind the shared connector API and Task 236 policy; preserve bounded connect/statement timeouts |
| `spire_remote_search_libpq_connect_with_session_timeouts` called from `node_registry.rs` | remote control identity during descriptor registration | sync shared SPIRE connector | same shared API and sanitized Task 236 categories |
| `traversal_replica::loopback_connection_config().connect(NoTls)` | dedicated connection back into the same coordinator for stale marking and owner-connection preflight | sync local side transaction, not coordinator-to-owner | retain only as an explicit local-only exception: Unix socket or loopback address, explicit local configuration, and a source comment/test proving the gate |

All public functions in `remote_transport.rs` converge on the first sink or the
sync callback/reaper sink. The inventory includes:

- reads: physical seed/head/expand, crown code, gateway routing, head-shard
  import/export, row and physical payload materialization;
- build and lifecycle: begin/stage/seal/abort handoff, publish, mark/apply
  retire, cancelled-generation reclaim, traversal-replica chunk;
- DML: remote insert, backlink, tombstone, prepared intent, transaction
  callbacks, and orphan reaping;
- registration: remote control-identity validation.

## Secret-resolution paths

`node_registry::resolve_conninfo_secret` validates the canonical secret name,
maps it to the existing provider environment key, and currently returns a raw
`String`. It is called by generation route construction, descriptor
registration, build/handoff phases, cancellation, publication, and retirement.
Those callers then pass only `&str` to transport, losing the stable secret
identity before the pool is selected.

Task 236 will replace that raw return value with a non-displayable resolved
secret value carrying:

1. the canonical secret reference;
2. the raw conninfo, exposed only to connector setup and the temporary legacy
   roster bridge;
3. a domain-separated SHA-256 credential/policy fingerprint.

No `Debug` or `Display` implementation may reveal the raw value. Errors expose
only a stable category, work-class context, and node/endpoint identity permitted
by the existing catalog contract.

## Current retention and exposure defects

1. `remote_transport` pool keys are strings containing the full raw conninfo
   (`lifecycle\u{1}{conninfo}` or `{conninfo}\u{1}{node_id}`). A rotated secret
   opens a new pool entry but leaves the old authenticated session resident
   until backend exit or an unrelated eviction.
2. `PhysicalOwnerRoute` stores resolved conninfo twice (`roster_conninfo` and
   optional `conninfo`), and `roster_spec_for_routes` serializes it into the
   session `ec_distann.roster` GUC. That makes password/TLS paths observable on
   generic GUC/status surfaces and forwards them to owner sessions.
3. async connect and non-database transport errors interpolate
   `tokio_postgres::Error`; write paths also include unsanitized server detail
   and hints. These must be classified before crossing the SQL boundary.
4. cancellation is always `NoTls`, even when the query connection will become
   TLS-protected.

The replacement pool key is `(work class, node/endpoint identity, canonical
secret reference, credential/policy fingerprint)`. Before reuse or insertion,
entries with the same endpoint plus secret reference and an older fingerprint
are removed and their driver tasks aborted. This bounds rotation invalidation
to the next connection use in that backend and keeps raw conninfo out of keys.

The production physical route will carry secret references and resolved secret
objects, not raw endpoint text in topology identity. The legacy operator roster
GUC remains accepted only for explicitly configured local test/compatibility
topologies; production catalog-backed routes must not synthesize or forward raw
conninfo through it.

## Frozen TLS and conninfo policy

The shared parser/connector will have an explicit caller policy. Task 236's
ec_distann production policy is:

| Input | Disposition |
|---|---|
| omitted `sslmode` | TLS required; equivalent to `require`, never plaintext |
| `sslmode=require` | TLS required, with libpq-compatible encryption-only semantics |
| `sslmode=verify-full` | TLS required; configured/system roots plus DNS/IP hostname verification |
| `sslmode=disable` | allowed only when explicitly present and every target is a Unix socket or loopback address |
| `sslmode=allow` / `prefer` | rejected; these modes permit or imply downgrade behavior |
| `sslmode=verify-ca` | rejected until the connector can implement CA verification without silently changing hostname semantics; operator hint says to use `verify-full` |
| `sslrootcert` | supported for `verify-full`; PEM must contain at least one valid trust anchor |
| `sslcert` + `sslkey` | supported only as a pair; PEM parsing and private-key file permissions fail closed |
| `sslpassword` / encrypted private key | rejected |
| `channel_binding=disable/prefer/require` | preserved; TLS streams provide `tls-server-end-point`, and `require` fails when binding cannot be supplied |
| `sslnegotiation=direct` and unimplemented libpq TLS options | rejected with the stable unsupported-option category; PostgreSQL negotiation remains supported |

Duplicate security options are rejected instead of using last-value-wins.
`verify-full` never falls back to an accept-any verifier. `require` never falls
back to plaintext. The shared SPIRE connector may temporarily select its
documented compatibility default through a separate caller policy, but parsing,
certificate loading, TLS stream/channel binding, sanitization, and connection
construction remain one implementation.

## Stable failure boundary

The connector result distinguishes at least:

- `secret_missing`, `secret_empty`, `secret_invalid`;
- `conninfo_parse_failed`, `tls_option_unsupported`, `plaintext_forbidden`;
- `ca_load_failed`, `client_cert_load_failed`, `client_key_load_failed`;
- `dns_failed`, `connect_timeout`, `connect_failed`;
- `tls_handshake_failed`, `certificate_invalid`, `hostname_mismatch`,
  `client_auth_failed`, `channel_binding_failed`;
- `connection_reset`.

Underlying parser, filesystem, rustls, and server error strings remain internal.
The SQL-facing mapping must not include raw conninfo, provider environment key,
password, certificate/key bytes or paths, remote row payload, or arbitrary
server detail/hints.

## Implementation slices

1. Extract the parser, rustls connector, sync/async connect, cancel connector,
   and channel-binding implementation into a shared `am` module with unit
   tests for the frozen policy and redaction.
2. Introduce the non-displayable resolved-secret/security-identity type and
   migrate all ec_distann route/request/pool interfaces. Remove conninfo from
   pool keys and catalog-backed roster/session identity.
3. Route every async, callback/reaper, registration, lifecycle, DML, and
   traversal-replica remote sink through the shared API; leave only the gated
   local side-transaction exception.
4. Add PG18 TLS/mTLS/rotation/negative-path probes and the required packet-local
   security matrix, leak inspection, and handshake versus pooled-reuse timings.
