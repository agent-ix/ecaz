---
task: 236
packet: 001-plan-and-connection-inventory
agent: Codex
role: coder
model: gpt-5
date: 2026-08-23
seq: 01
---

# Task 236 secure transport and secret resolution plan

This packet requests review of Task 236's completed planning and connection
inventory. The implementation base is
`308eea630beb0f619bdba71c62686c5ca4140174`.

FR-079's Task-214 F10 gap records that ec_distann remains hardwired to `NoTls`
even though generation catalogs persist secret references and NFR-014 requires
resolved libpq security settings. Task 236 inventories every async and sync
connection path, then extracts or reuses the existing SPIRE rustls-backed
tokio-postgres connector instead of defining a second TLS parser/verifier.

The contract preserves supported sslmode, CA, hostname, client certificate/
key, and channel-binding behavior; prevents verified-to-plaintext downgrade;
keys/invalidate pooled connections across credential/policy changes; sanitizes
failure categories; and keeps raw conninfo, keys, payloads, and server details
out of catalogs, SQL diagnostics, EXPLAIN, logs, manifests, and artifacts.

Please review connector reuse, supported/rejected option policy, secret
lifetime, pool-rotation identity, negative TLS matrix, and zero-exposure audit.
Task 228 is sequenced afterward so its real-network evidence includes the
production security overhead.

The durable inventory is
[`artifacts/connection-inventory.md`](artifacts/connection-inventory.md). It
enumerates the async pool/connect/cancel sink, sync registration and DML
callback/reaper sinks, the explicitly local traversal-replica side-transaction
exception, every work class converging on those sinks, current secret
resolution, raw-conninfo retention defects, the replacement pool/rotation
identity, the frozen supported/rejected TLS option policy, stable failure
categories, and implementation slices.

The inspection found two additional security defects that the original request
did not make explicit: raw conninfo is embedded in pool keys, and resolved
conninfo is serialized into the legacy session roster GUC. Both are now in the
Task 236 implementation scope.

This is planning/inventory only. No tests were run.
