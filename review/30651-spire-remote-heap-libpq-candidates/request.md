# Review Request: SPIRE Remote Heap Libpq Candidates

## Scope

This packet reviews commit `6d004304 Resolve SPIRE remote heap candidates over libpq`.

The slice adds the first origin-node heap resolution executor path:

- adds `ec_spire_remote_search_libpq_executor_heap_candidates(...)`.
- sends a second libpq request to the remote node's existing
  `ec_spire_remote_search_local_heap_candidates(...)` endpoint.
- decodes remote heap block/offset on the origin node, normalizes remote `node_id` back to the
  coordinator descriptor id, and validates the returned candidate batch before exposing rows.
- marks returned rows with `heap_lookup_owner = 'origin_node_row_locator'`.
- extends the existing search libpq loopback fixture to assert one resolved remote heap candidate.

## Validation

Focused PG18 coverage:

```text
cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty
git diff --check
```

The PG18 test passed. The fixture verifies connection open, nonempty remote candidate receive,
origin-node heap candidate resolution, descriptor-node id normalization, and a positive heap offset.

## Review Notes

- This exposes resolved remote heap coordinates as executor output; it does not yet compose those rows into
  `ec_spire_remote_search_coordinator_result_summary(...)`.
- The remote heap lookup is still loopback-based. Distinct multi-cluster validation remains a later item.
