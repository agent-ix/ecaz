# Review Request: SPIRE Local Store Read-Overlap Harness

Code checkpoint: `975e8d83` (`Add SPIRE local-store read overlap harness`)

## Scope

- Advances Phase 12.8 by adding
  `ec_spire_index_scan_local_store_read_overlap_harness(index_oid, query)`.
- Extends the scan-placement diagnostic observer with per-store:
  - prefetched object bytes;
  - read-batch count;
  - delta-decode count.
- Keeps `ec_spire_index_scan_placement_snapshot(...)` unchanged to avoid
  widening the already-large SQL row shape.
- Adds a PG18 multi-store SQL fixture that builds a two-store index, inserts a
  post-build delta row, then asserts the harness reports two touched store
  groups, one read batch per touched group, positive object bytes, and one
  selected delta decode.
- Documents the harness in diagnostics/design docs and marks the Phase 12.8
  read-overlap harness row complete.

## Validation

- `git diff --check 975e8d83^ 975e8d83`
- `cargo fmt --check`
- `cargo test --no-default-features --features pg18 collect_scan_placement_diagnostics_counts_routed_store_rows --lib`
- `cargo test --no-default-features --features pg18 prefetch_store_object_read_groups_prefetches_every_store_before_scoring --lib`
- `cargo pgrx test pg18 test_ec_spire_multistore_read_overlap_harness_sql`

Packet-local logs are under `artifacts/`; see `artifacts/manifest.md` for
commands and key result lines.

## Review Focus

- Confirm the new narrow harness is the right surface instead of widening
  `ec_spire_index_scan_placement_snapshot(...)`.
- Confirm `read_batch_count` is correctly defined as one current backend
  prefetch/read group per touched `(node_id, local_store_id)`, not a claim of
  concurrent multi-store execution.
- Confirm `delta_decode_count` correctly tracks selected delta object reads,
  preserving the reuse contract covered in packet `30947`.
