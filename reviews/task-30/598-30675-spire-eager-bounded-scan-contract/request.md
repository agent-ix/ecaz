# Review Request: SPIRE Eager Bounded Scan Contract

Code checkpoint: `47559184` (`Document SPIRE eager bounded scan contract`)

## Scope

- Completes Phase 10.2 by choosing the current eager bounded AM scan shape.
- Adds ADR-056, which records that `amrescan` owns snapshot/object-store access,
  routing, candidate collection, and heap rerank, while `amgettuple` drains a
  pre-ranked cursor.
- Documents the memory ceiling as bounded route frontier plus bounded candidate
  cursor, and the latency tradeoff as first-tuple work in `amrescan`.
- Records that streaming `amgettuple` is deferred until a separate ownership
  and failure-mode ADR is accepted.
- Marks the Phase 10.2 checklist complete against ADR-056 and existing
  forward-only AM behavior.

## Validation

- `git diff --check`
- `cargo test --no-default-features --features pg18 scan_opaque --lib`

## Review Focus

- Confirm eager bounded scan is the right Phase 10 contract before optimizing
  heap rerank, multi-store reads, and remote dispatch.
- Confirm ADR-056 states the snapshot/object-store ownership boundary clearly
  enough to block accidental streaming behavior in `amgettuple`.
- Confirm Phase 10.2 can be considered complete without new scan callback code,
  because the implementation already enforces forward-only cursor drain.
