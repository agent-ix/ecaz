---
topic: spire-classify-centroid-helper
agent: coder2
role: coder
model: GPT-5
date: 2026-05-11
stage: task-30-phase11-stage-d-adr069
status: open
---

# Review Request: SPIRE Centroid Classification Helper

## Scope

This packet lands the ADR-069 coordinator-side classifier helper:

```sql
SELECT * FROM ec_spire_classify_centroid(
  ARRAY[1.0, 0.0]::real[],
  'documents_embedding_idx'::regclass
);
```

The helper returns:

- `node_id`
- `centroid_id`
- `epoch`

Implementation notes:

- The SQL wrapper opens and validates an `ec_spire` index, then calls an
  internal AM classifier.
- The classifier reads the active epoch, object manifest, and placement
  directory without using the legacy local-heap materialization gate.
- It walks local routing objects by max inner product, tie-breaking by
  `centroid_index` then child pid.
- It returns the selected leaf placement metadata directly, using the selected
  leaf pid as the active-epoch `centroid_id`.
- Non-leaf routing objects are required to remain local; remote leaves are
  supported and are the path covered by the PG18 test.
- The Phase 11 task tracker marks only the classifier helper as complete.

This does not implement coordinator-routed INSERT, remote 2PC, placement-row
registration during INSERT, UPDATE/DELETE forwarding, or PK-keyed SELECT.

## Validation

Packet-local logs are in `artifacts/`.

- `cargo test classify_centroid --lib`
- `cargo fmt --check`
- `git diff --check`

## Review Focus

- Confirm `centroid_id = selected leaf pid` is the right durable interpretation
  for the ADR-069 placement directory.
- Confirm the helper bypasses only the superseded local-heap materialization gate
  and still validates active routing metadata.
- Confirm recursive routing behavior is acceptable for the INSERT path: local
  routing objects are traversed, remote leaves are returned, and remote non-leaf
  routing objects fail closed.
