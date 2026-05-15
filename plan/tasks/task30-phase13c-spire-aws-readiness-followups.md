# Task 30 Phase 13c: SPIRE AWS Readiness Follow-ups

Status: implemented locally; review pending
Owner: coder1 / SPIRE AWS verification track
Priority: P1 before Phase 13b AWS execution

## Context

Packet `764` reviewer feedback (`2026-05-15-01-reviewer`) found two
AWS-blocking production-boundary gaps after Phase 12c closeout:

- production remote connection paths still used `NoTls`, so resolved
  `conninfo_secret_name` values could not enforce AWS TLS policy; and
- the coordinator-routed PK SELECT primitive skipped the read schema-drift
  fingerprint guard that vector CustomScan reads already use.

Phase 13c lands the local fixes required before spending AWS time. It does not
provision AWS infrastructure and does not optimize the two-stage candidate/heap
fanout shape.

## 13c.1: Remote libpq TLS connector (P1)

- [x] Route every production SPIRE remote open through shared sync/async
  helpers instead of direct `postgres::NoTls` / `tokio_postgres::NoTls`.
- [x] Preserve local/dev non-TLS operation when the resolved conninfo uses
  `sslmode=disable` or a non-TLS local socket.
- [x] Support AWS/prod conninfo values that require TLS, including
  `sslmode=require` and `sslmode=verify-full` with `sslrootcert`.
- [x] Preserve `target_session_attrs` and other tokio-postgres-supported
  connection parameters while stripping only the TLS parameters handled by the
  SPIRE connector layer.
- [x] Route cancel requests through the same TLS policy as the connection they
  cancel.

Acceptance:

- [x] `rg` finds no direct production `NoTls` call sites outside the shared
  TLS helper under `src/am/ec_spire/coordinator/remote_candidates/`.
- [x] `sslmode=disable` continues to use non-TLS local connections.
- [x] `sslmode=require` opens an encrypted connection when the server supports
  TLS.
- [x] `sslmode=verify-full sslrootcert=...` verifies the remote certificate
  chain and hostname.

## 13c.2: PK SELECT read schema-drift guard (P1)

- [x] Run `validate_read_shape_fingerprints_before_remote_dispatch` in
  `coordinator_select_remote_tuple_payload` before remote SQL dispatch.
- [x] Treat PK SELECT as strict for v1 because the primitive has no degraded
  consistency-mode argument.
- [ ] Add or cite focused coverage that exercises coordinator-only,
  remote-only, and both-sides PK SELECT drift before remote dispatch.

Acceptance:

- [ ] PK SELECT with stale descriptor fingerprints fails with
  `schema_drift` before opening the remote SELECT.
- [ ] Existing vector-read drift behavior remains unchanged.

## Deferred Follow-ups

- Production read fanout still performs separate candidate and heap remote
  connection/identity probes. Measure this in Phase 13b before deciding whether
  to combine endpoints or keep a per-dispatch session across phases.
- SPIRE's large `include!` module topology remains a maintainability issue,
  but it is not an AWS execution blocker once TLS and PK SELECT drift are fixed.
