# Task 111h / 005 - Packed Rerank Group Integration Checkpoint

## Scope

Code commit: `6edd28d1449a0148e676b8181f5dc1bcbf362d77`

This checkpoint switches the current compact index-side rerank storage path from
the legacy `0x2A` heap-TID sidecar payload map to packed scorer-width rerank
groups:

- Build writes `0x2B` group headers plus `0x2C` payload continuation segments.
- Postings store the owning group header TID directly.
- Group headers chain through `next_group_tid`; payload continuations chain
  through `next_segment_tid`.
- Scan loads unique group headers, reassembles each group payload once, and
  locates candidates through the group-local heap-TID slots instead of rebuilding
  a per-query heap-TID payload map.
- Insert prepends a single-entry packed group and links it into the list.
- Vacuum tombstones matching group slots in the group header deleted bitmap.
- IVF on-disk format is bumped to v5, with docs, fixtures, upgrade matrix, and
  size/version assertions updated. v4 is rejected as a legacy incompatible
  layout.

## Non-Claims

This packet is not benchmark evidence. It contains correctness/static validation
for the packed group integration only.

The following Task 111h items remain open:

- Full `ecaz bench suite` latency/recall/storage sweep.
- Legacy `0x2A` benchmark baseline before deleting or demoting that path.
- Table-owned persisted compact payload storage and evidence.
- Eliminating the remaining survivor-payload batch slab copies in the scan path.
- EXPLAIN/admin counters for the active storage layout.
- Broader PG18 fixtures for mixed old/new, update, delete/vacuum, and concurrent
  insert/read cases.

## Validation

Packet-local logs are under `artifacts/` and summarized in
`artifacts/manifest.md`.

- `cargo-check-pg18.log`: `cargo check --no-default-features --features pg18`
  passed.
- `cargo-test-rerank-group.log`: 3 passed.
- `cargo-test-data-page-chain.log`: 1 passed.
- `cargo-test-on-disk-ivf-metadata.log`: 4 passed.
- `cargo-test-size-of-assertions.log`: 13 passed.
- `cargo-test-upgrade-matrix.log`: 2 passed.
- `cargo-test-index-quant-formats.log`: 1 passed.
- `cargo-test-index-placement.log`: 10 passed.
- `cargo-test-coarse-rerank.log`: 23 passed.

## Review Focus

- Verify list/group flush semantics in build: logical scorer-width groups,
  list-boundary flushes, and final-tail handling.
- Verify direct posting-to-group-header lookup in scan replaces the old
  heap-TID payload map rebuild.
- Verify insert and vacuum behavior for packed group headers, especially
  singleton live inserts and deleted-slot bitmap rewrites.
- Verify the v5 format bump, fixture, docs, matrix, and rejection policy satisfy
  NFR-016 for this incompatible layout change.
