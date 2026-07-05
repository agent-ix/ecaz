---
topic: spire-placement-batch-registration
agent: coder2
role: coder
model: GPT-5
date: 2026-05-11
stage: task-30-phase11-stage-d-adr069
status: open
---

# Review Request: SPIRE Placement Batch Registration

## Scope

This packet lands the ADR-069 bulk-load post-write registration primitive:

```sql
SELECT ec_spire_register_placement_batch(
  index_oid => 'documents_embedding_idx'::regclass,
  entries   => ARRAY[
    ROW($pk_value, $node_id, $centroid_id, $served_epoch, $source_identity)
      ::ec_spire_placement_entry
  ]
);
```

Changes:

- Add composite type `ec_spire_placement_entry`.
- Add SQL function
  `ec_spire_register_placement_batch(index_oid oid, entries ec_spire_placement_entry[])`.
- Insert batch entries into the coordinator-local `ec_spire_placement` table.
- Return the inserted row count as `bigint`.
- Rely on the placement table constraints from packet 30817 for non-empty
  `pk_value`, positive `node_id`, non-negative `centroid_id`, positive
  `served_epoch`, 16-byte `source_identity`, and primary-key uniqueness.
- Update the Phase 11 task tracker to mark only this primitive complete.

This does not implement coordinator-routed INSERT, remote 2PC, write hooks, or
remote forwarding.

## Validation

Packet-local logs are in `artifacts/`.

- `cargo test placement_batch --lib`
- `cargo fmt --check`
- `git diff --check`

## Review Focus

- Confirm the composite entry shape matches ADR-069.
- Confirm strict insert semantics are acceptable for the v1 primitive; duplicate
  `(index_oid, pk_value)` rows currently fail through the placement table
  primary key instead of silently upserting.
- Confirm the function belongs in both bootstrap and `0.1.0--0.1.1` upgrade
  SQL for the current task branch.
