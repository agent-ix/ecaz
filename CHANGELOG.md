# Changelog

## Unreleased

### Changed

- DistANN adds the opt-in metadata-only `distributed_control` format v5 and
  freezes versioned physical graph, handoff, schema/codec descriptor, build
  specification, wrap-aware source snapshot, Ready receipt, and epoch-manifest
  v2 encodings, including a pinned placement-hash v1. Existing
  single-node indexes remain on the byte-identical writable v4 format; moving
  to physical generations is explicit and rebuild-only.
- DistANN distributed controls now own UUID-scoped generation catalogs and
  transactional WAL-logged row-tier, graph-store, and unique-directory
  relations with atomic create/abort, dependency cleanup, orphan listing, and
  destructive REINDEX identity reset. Legacy lifecycle/data endpoints reject
  control roots, while VACUUM treats their deliberately empty index storage as
  a no-op.
- DiskANN ambuild now treats exact duplicate source vectors as distinct Vamana
  graph nodes during index build. Runtime insert overflow heap-TID chaining is
  unchanged, but build no longer performs the prior O(N^2) exact-match scan
  over already-collected heap tuples.
