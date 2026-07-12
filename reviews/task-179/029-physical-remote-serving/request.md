---
agent: codex
role: coder
model: gpt-5
date: 2026-07-12
seq: 01
---

# Review request — Packet 029 physical remote serving

This checkpoint makes a Published physical generation readable across every
owner in its immutable build roster.

## Commit

- `6f9b98bfa` — serve physical generations across owners.

## Behavior

- Physical read RPCs resolve the exact retained `Published` or `Retired`
  generation by the v2 manifest fingerprint, never by a participant active
  pointer.
- The coordinator loads immutable build bindings, collects bounded responses
  from every owner's seed RPC, partitions every hop batch with the manifest's
  placement function, and reassembles expansion rows in request order.
- The FR-079 physical `ec_distann_expand_nodes(regclass, ...)` overload validates
  placement, reads the selected immutable graph relation, scores embedded
  neighbor codes, and exact-reranks from the co-placed frozen row tier.
- The physical `ec_distann_materialize_row_payloads(regclass, ..., smallint[],
  bytea)` overload accepts attnums only. The owner validates the exact frozen
  schema fingerprint and resolves `typsend` identities from its row-tier
  catalog instead of accepting caller-selected SQL function names.
- CustomScan keeps local frozen CTIDs local and reconstructs remote-owned rows
  from binary payloads, including projected/qual attributes.
- Every endpoint materializes its complete response before returning an SQL
  iterator, so a failed batch cannot expose a partial prefix.

## Live evidence

The focused PG18 three-owner fixture now forces `EcDistannDistributedScan`,
returns all 30 frozen rows after physical publication, and proves the served
identities cover roster ordinals 0, 1, and 2. The two remote owners remain
separate participant control indexes and physical generation relations, reached
through pooled loopback transport.

Validation and provenance are in `artifacts/manifest.md`.

## Explicit remaining work

- This packet's seed response is bounded, but each owner still discovers seeds
  with an O(N) immutable graph scan. Persisted bounded head state and exact
  manifest binding remain required before closeout.
- The fixture is process-local loopback topology, not the required real three
  PostgreSQL-instance fixture.
- The mandatory 10k/50k/100k `ecaz bench suite` A/B recall, latency, and storage
  evidence remains open.

Leaving this request open for outside review.
