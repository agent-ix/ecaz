# Review Request: SPIRE ReadStream Local Fetch

- Code commit: `d677a9a5` (`Batch SPIRE relation prefetch with ReadStream`)
- Tracker commit: `58349306` (`Record SPIRE ReadStream local fetch progress`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation, Phase 4 local placement
- Agent: coder1

## Summary

This checkpoint turns the scan prefetch path from per-object prefetch calls into
a batched object-reader prefetch contract:

- `SpireObjectReader` now exposes `prefetch_objects(&[SpirePlacementEntry])`
  with a default per-object fallback;
- scan scheduling collects all selected leaf and delta placements across store
  groups and sends them to the object reader in one batch before scoring;
- relation-backed object stores group placements by `(local_store_id,
  store_relid)`, dedupe object tuple block numbers, and order the groups
  deterministically;
- PG18 builds use `read_stream_begin_relation` with the existing
  `BlockSequencePrefetchState` callback to drain each store relation's object
  blocks before candidate scoring;
- non-PG18 builds retain the `PrefetchBuffer` fallback.

This is the Phase 4 local fetch surface for PostgreSQL async read-ahead. It is
not backend-thread parallelism, and it does not claim production multi-NVMe
performance by itself.

## Review Focus

1. Confirm that `SpireObjectReader::prefetch_objects` is the right boundary for
   future store-local fetch improvements.
2. Check that relation-backed grouping validates local node, placement state,
   and store membership before issuing read-ahead.
3. Verify that deduping block numbers per store relation is safe for multiple
   object tuples on the same page.
4. Check that the PG18 `ReadStream` usage mirrors existing `ec_ivf`/common
   stream patterns and always ends the stream.
5. Confirm that the tracker wording is appropriately precise: async read-ahead
   within one backend, not worker-thread parallelism.

## Validation

- `cargo test relation_object_prefetch_groups --lib`
- `cargo test prefetch_store_object_read_groups --lib`
- `cargo test collect_quantized_routed_probe_candidates_reads_hash_routed_two_store_build --lib`
- `cargo fmt --check`
- `git diff --check`
- `cargo pgrx test pg18 test_ec_spire_populated_build_hash_routes_logical_store_set`

PG17 was not run; the load-bearing runtime change is specifically PG18
`ReadStream`, while non-PG18 retains the existing prefetch fallback.

## Notes

This packet follows `30532`, which resolved placements once before prefetch.
That earlier shape made this checkpoint small: relation-backed readers can now
consume the already-resolved placement batch directly rather than rebuilding
lookups during prefetch.
