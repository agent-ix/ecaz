---
task: 198
packet: 001-contract-and-format
role: coder
status: review_requested
date: 2026-07-23
seq: 1
---

# Review request: traversal-replica contract and format

This checkpoint defines the correctness boundary before the faithful
prototype:

- FR-084 specifies the coordinator-only derived-object role, heap/directory
  layout, canonical content digest, state machine, epoch selection, complete
  fallback, side-transaction mutation invalidation, operator surface, and
  acceptance criteria.
- `ec_distann_traversal_replica` and its per-owner coverage table persist the
  immutable identity, physical OIDs, state, counts, digests, and cost fields.
- `TraversalReplicaContentHasher` binds sorted owner/vec_id rows, exact graph
  bytes, exact-vector bytes, active fingerprint, descriptor, shape, and global
  cardinality. It rejects duplicate/out-of-order rows, wrong owners, malformed
  vector shape, non-finite values, and incomplete streams.

The storage remains derived and payload-free. Owner generations, the FR-082
active pointer, and owner-side final materialization remain authoritative.
There is no scan/build behavior in this checkpoint.

Focused PG18 unit evidence is in
`artifacts/pg18-unit.log`: 3 passed, 0 failed. The artifact manifest records
the exact command, head, timestamp, SHA-256, and scope.

Please review the canonical ordering/digest domain, catalog invariants,
Ready/Stale/Retiring transitions, loopback control-transaction contract, and
whether any lifecycle state needed by the faithful prototype is missing.
