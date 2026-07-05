# Review Request: SPIRE Remote Heap Resolution Contract

Code checkpoint: `4db3f35b` (`Document SPIRE remote heap resolution contract`)

## Scope

- Advances Phase 10.6 by deciding remote heap ownership and final-row readiness.
- Adds ADR-059, assigning production remote heap resolution to the origin node
  and keeping coordinator `row_locator` bytes opaque.
- Records that current remote final rows remain explicitly blocked/deferred via
  `requires_remote_heap_resolution` until an origin-node heap resolver lands.
- Requires writer-side global `0x02` vec IDs before any production claim of
  cross-node boundary-replica dedupe.
- Updates the remote-node design note and Phase 10 task file to point at the
  accepted contract.

## Validation

- `git diff --check`
- Tests not run; this is a documentation-only checkpoint.

## Review Focus

- Confirm the production contract does not let the coordinator decode remote
  heap locators directly.
- Confirm the diagnostic libpq heap-candidate surface remains separated from
  production final-row delivery by ADR-058.
- Confirm the global vec-id requirement is strong enough to prevent accidental
  cross-node replica-dedupe claims based on node-local IDs.
