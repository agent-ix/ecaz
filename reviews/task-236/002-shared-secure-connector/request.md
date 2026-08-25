---
task: 236
packet: 002-shared-secure-connector
agent: Codex
role: coder
model: gpt-5
date: 2026-08-24
seq: 01
---

# Task 236 shared secure connector implementation

This packet requests outside security/code review of the shared connector
implementation at `8c535390dea709d5000a2ce376c3cae8d812ebd6`.
It does not request Task 236 closeout: the live PG18 TLS/mTLS/rotation matrix,
leak inspection, and handshake-versus-pooled benchmark evidence remain packet
003 work.

## Implementation checkpoints

- `b6511251f` extracts one rustls connector/parser for SPIRE and DistANN and
  uses `tokio-postgres-rustls` channel binding support.
- `94875f23d` migrates all async DistANN query and cancellation connections to
  the strict policy; local fixtures opt out explicitly with
  `sslmode=disable`.
- `d368f491d` migrates registration, DML callback, and prepared-xact reaper
  connections to the same strict blocking connector and DistANN timeouts.
- `198b19d59` removes raw conninfo from pool keys and evicts an older
  credential/policy generation on next use of the same endpoint/work identity.
- `f5b18cadd` introduces a non-displayable resolved-secret value with separate
  stable secret-identity and credential/policy fingerprints.
- `4db6d4c1e` stores typed secrets in physical routes and forwards only
  `secret:NAME` topology references in catalog-backed session rosters; raw
  roster conninfo remains a documented legacy local-fixture path.
- `fc792e581` replaces driver/server error interpolation with stable sanitized
  categories, retaining only a bounded numeric vec_id for the missing-owned-
  record correctness path.
- `8c535390d` centralizes the remaining plaintext side transaction behind a
  tested explicit-loopback-only gate and removes `NoTls` from the traversal
  replica module.

## Frozen behavior implemented

DistANN defaults omitted `sslmode` to TLS-required, supports `require` and
`verify-full`, preserves supported channel-binding modes, supports CA and
paired client certificate/private-key inputs, and rejects downgrade-capable,
ambiguous, or unsupported security options. Explicit `sslmode=disable` is
accepted only when every target is loopback or a Unix socket. Query
cancellation uses the same TLS configuration as the pooled query stream.

Raw conninfo and server error text are absent from pool keys, typed secret
debug output, production session rosters, connector errors, and remote DML/
lifecycle/read failures. Rotation changes the security fingerprint and removes
the superseded pooled session on the next matching endpoint/work use.

## Validation

See [`artifacts/manifest.md`](artifacts/manifest.md). The packet-local PG18
results are:

- compile check: pass;
- shared TLS policy: 8/8 pass;
- DistANN transport: 15/15 pass;
- typed physical route/roster: 3/3 pass;
- resolved-secret identity/redaction: 1/1 pass.

Please focus review on TLS policy equivalence, hostname/channel-binding
semantics, certificate/key handling, rotation eviction, raw-secret lifetime,
sanitized failure boundaries, the `secret:NAME` roster bridge, and the scope of
the explicit loopback plaintext exception.
