# SPIRE Libpq Identity Cache Contract

Status: Phase 11 Stage C design contract
Date: 2026-05-09
Scope: validated remote endpoint identity caching for SPIRE libpq fanout

## Purpose

Stage B intentionally validates `ec_spire_remote_search_endpoint_identity()` on
every libpq dispatch before compact candidate or remote heap candidate receive.
Stage C may cache that validation to avoid a per-dispatch identity round trip,
but the cache must remain an optimization only. It must not make a stale
descriptor, rebuilt remote index, extension-version skew, or endpoint
fingerprint mismatch look ready.

## Required Cache Key

The minimum key is:

```text
coordinator_index_oid
node_id
remote_index_regclass
remote_index_identity
served_epoch
```

The cache entry must also bind the descriptor generation and the live endpoint
identity fields that feed the v1 profile fingerprint:

```text
descriptor_generation
protocol_version
extension_version
opclass_identity
storage_format
assignment_payload_format
quantizer_profile
scoring_profile
profile_fingerprint
```

`remote_index_identity` is the descriptor-side bytea identity. In v1 it must be
equal to the live endpoint `profile_fingerprint` bytes before the entry can be
inserted or reused.

Raw conninfo is not a cache key and must not be stored in the identity cache.
The cache may store `conninfo_secret_name` only as a diagnostic binding to the
descriptor generation; secret resolution remains a separate executor step.

## Reuse Rules

A cached validation entry is reusable only when all of these are true:

- the coordinator index OID, `node_id`, remote index regclass, descriptor
  generation, descriptor `remote_index_identity`, and requested served epoch
  match the key exactly;
- the descriptor remains `active` or `draining`;
- `last_served_epoch >= requested_epoch` and
  `min_retained_epoch <= requested_epoch`;
- the descriptor extension version still matches the coordinator extension
  version;
- the endpoint protocol, opclass identity, storage format, assignment payload
  format, quantizer profile, scoring profile, and profile fingerprint match the
  cached validation entry exactly.

If any predicate fails, the executor must treat the entry as a miss and run the
live endpoint identity preflight before receive.

## Invalidation Triggers

The cache must invalidate the affected node entry on:

- remote-node descriptor register/update/delete for that coordinator index and
  node;
- descriptor generation change;
- descriptor `remote_index_identity` change;
- served epoch or retained epoch-window change;
- remote endpoint live fingerprint mismatch;
- extension version change;
- opclass identity change;
- storage format or assignment payload format change;
- quantizer or scoring profile change;
- remote index regclass change;
- local extension upgrade.

Any dispatch that observes a different live `profile_fingerprint` than the
descriptor identity must invalidate the entry and fail closed in strict mode
with `endpoint_identity_mismatch`. In degraded mode it must report
`skip_node` with `next_blocker = remote_endpoint_identity`. It must not
silently reseat the descriptor identity from the live endpoint.

## Resource Bounds

The first Stage C cache should be backend-local and bounded. A small per-query
or per-backend map keyed by the contract above is acceptable; an unbounded
global shared cache is not. Any later shared cache needs an explicit memory cap,
lock-order contract, and invalidation path from descriptor writes.

## Test Matrix Before Enabling Cache Reuse

Cache reuse is not production-ready until PG18 coverage proves:

- ready descriptor plus matching endpoint reuses the entry;
- descriptor generation change invalidates the entry;
- descriptor identity change invalidates the entry;
- served epoch advance invalidates or misses the entry;
- stale epoch and retention gap do not reuse;
- extension-version skew does not reuse;
- live fingerprint mismatch invalidates and reports
  `endpoint_identity_mismatch`;
- remote heap candidate receive uses the same cache decision as compact
  candidate receive;
- strict mode fails closed and degraded mode skips with exact diagnostics.
