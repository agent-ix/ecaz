# Review Request: SPIRE Local Store Read Scheduling Contract

Code checkpoint: `839b0b4f` (`Document SPIRE local store read scheduling`)

## Scope

- Advances Phase 10.4 by making the local multi-store read-overlap contract
  explicit.
- Adds ADR-057, accepting `(node_id, local_store_id)` as the scheduling unit
  and PostgreSQL relation prefetch/read-stream as the Phase 10 overlap
  primitive.
- Records that object decoding and candidate scoring remain sequential inside
  one backend until a later ADR covers true parallel store execution.
- Links the existing local multistore design note to ADR-057.
- Marks the Phase 10.4 local-store overlap/sequential-limitation checklist
  item complete.

## Validation

- `git diff --check`
- Tests not run; this is a documentation-only checkpoint.

## Review Focus

- Confirm ADR-057 accurately describes the current implementation boundary:
  grouped store reads, prefetch before scoring, sequential decode/score.
- Confirm the design does not overclaim multi-NVMe parallelism.
- Confirm the future-work gate is specific enough for true parallel store
  execution: ownership, cancellation, failures, deterministic merge, and
  hardware measurement.
