# Review Request: SPIRE Indexed Store Lookup

Code checkpoint: `58c2f08e` (`Index SPIRE local store lookups`)

## Scope

- Advances Phase 10.4 by replacing repeated linear local-store lookup with
  explicit index maps.
- Adds `local_store_id -> stores[index]` lookup for in-memory
  `SpireLocalObjectStoreSet`.
- Adds `(local_store_id, store_relid) -> stores[index]` lookup for relation
  backed `SpireRelationObjectStoreSet`, including prefetch-group dispatch.
- Preserves the existing `Vec` storage ownership and relation close order; the
  maps only hold stable indexes into that vector.
- Marks the Phase 10.4 indexed lookup checklist item complete.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test --no-default-features --features pg18 local_object_store_set_indexes_non_contiguous_store_ids --lib`
- `cargo test --no-default-features --features pg18 relation_object_prefetch_groups --lib`
- `cargo test --no-default-features --features pg18 local_object_store --lib`
- `cargo test --no-default-features --features pg18 collect_quantized_routed_probe_candidates_reads_hash_routed_two_store_build --lib`

## Review Focus

- Confirm the index maps cannot drift from the backing `stores` vector after
  construction.
- Confirm relation-backed lookup still keys on both `local_store_id` and
  `store_relid`, so stale or mismatched placements fail closed.
- Confirm this closes only the indexed lookup item; read-overlap and richer
  per-store/top-graph I/O diagnostics remain open Phase 10.4 work.
