# Review Request: SPIRE Remote Heap Libpq Candidates

## Scope

This packet reviews these related commits:

- `6d004304 Resolve SPIRE remote heap candidates over libpq`
- `efd0a5cb Summarize SPIRE remote heap libpq candidates`

The slice adds the first origin-node heap resolution executor path:

- adds `ec_spire_remote_search_libpq_executor_heap_candidates(...)`.
- sends a second libpq request to the remote node's existing
  `ec_spire_remote_search_local_heap_candidates(...)` endpoint.
- decodes remote heap block/offset on the origin node, normalizes remote `node_id` back to the
  coordinator descriptor id, and validates the returned candidate batch before exposing rows.
- marks returned rows with `heap_lookup_owner = 'origin_node_row_locator'`.
- adds `ec_spire_remote_search_libpq_executor_heap_candidate_summary(...)` so operators can inspect
  returned remote heap candidate counts and result source without consuming every row.
- extends the existing search libpq loopback fixture to assert one resolved remote heap candidate.

## Validation

Focused PG18 coverage:

```text
cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty
git diff --check
```

The PG18 test passed. The fixture verifies connection open, nonempty remote candidate receive,
origin-node heap candidate resolution, descriptor-node id normalization, a positive heap offset, and
the heap candidate summary reporting `result_source = 'remote_heap_candidates'` with `status = 'ready'`.

## Review Notes

- This exposes resolved remote heap coordinates as executor output; it does not yet compose those rows into
  `ec_spire_remote_search_coordinator_result_summary(...)`.
- The remote heap lookup is still loopback-based. Distinct multi-cluster validation remains a later item.
