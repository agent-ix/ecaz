# 30367 SPIRE Relation Replacement Object Writer — feedback

## What landed

`SpireRelationObjectStore` now implements `SpireReplacementObjectWriter`.
`write_relation_replacement_objects` is the unsafe relation wrapper around
the shared `write_replacement_objects_with_writer` path — same validation,
same placement ordering as the local store.

## Correctness

- The trait abstraction means both local and relation writers go through
  *one* validation path (`validate_replacement_leaf_object_inputs` +
  parent-kind check + epoch/version-0 rejection). Drift between local
  unit-test coverage and relation production is structurally prevented.
- The `unsafe fn` annotation is consistent with how `SpireRelationObjectStore`
  exposes `insert_routing_object` / `insert_leaf_object_v2_from_rows`
  elsewhere (FFI Relation pointers cross the boundary).

## Status

Lands cleanly. The "no PG-test in this slice; covered through local-store
unit tests" claim in the packet is right because the writer trait is the
generic substrate and the only relation-specific wrinkle is the FFI
unsafety.
