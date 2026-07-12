---
agent: codex
role: coder
model: gpt-5
date: 2026-07-11
seq: 01
---

# Review request — Packet 020 Published physical-generation read

This is the first end-to-end replacement of the legacy control-index read path:
the metadata-only logical index now resolves and reads its Published physical
generation without materializing graph pages back into the control index.

## Commit

- `21fd1a6f` — read Published physical generations through CustomScan.

## Resolution and lifetime

- Planner eligibility now selects CustomScan for a v5 distributed-control index
  even in the single-owner degenerate topology; legacy multi-node eligibility is
  preserved.
- Executor resolves `ec_distann_active_epoch` to build id/fingerprint, registers
  an exact `ScanTokenGuard`, re-reads the pointer, validates the Published
  generation descriptor identity/digest, and opens row/graph relations under
  AccessShare locks.
- Executor state owns the token and relation guards through rescans and teardown.

## Search and materialization

- The reader restores the immutable codec artifact, scores generation-local
  graph records for a bounded seed set, and drives the shared FR-081 beam
  orchestration through a graph-relation expander.
- Expansion batch-resolves requested vec ids, decodes physical-v1 records,
  verifies graph-column/record/row-TID agreement, scores embedded neighbor
  codes, and exact-reranks the frozen vector fetched through the record's row-tier
  CTID.
- Result materialization fetches the immutable row-tier tuple into an
  estate-managed slot and copies it to the CustomScan projection slot.

## Live evidence

The real-backend multi-epoch PG18 fixture asserts the query plan contains
`EcDistannDistributedScan` and that a query nearest `[1,0,0,0]` returns the
expected frozen row after first-epoch publication. It then continues through
the second-epoch predecessor-CAS path.

Validation and provenance are in `artifacts/manifest.md`.

## Explicit next work

- Seed selection currently scans physical search codes because the generation
  schema has no persisted FR-080 head-sample relation. Persisted bounded head
  state must replace this O(N) seed pass before latency benchmarks.
- This checkpoint is the single-local-owner physical path. Multi-owner routing,
  owner-side physical expand/materialize endpoints, and the real three-node
  fixture remain required.
- T4b retirement/reclaim and required A/B measurements remain open.

Leaving this request open for outside review.
