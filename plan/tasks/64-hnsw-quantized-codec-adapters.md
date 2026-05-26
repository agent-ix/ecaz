# Task 64: HNSW Quantized Codec Adapters

Status: **proposed**

Companion task to Task 63. Extract a narrow HNSW-local quantized codec adapter
surface so HNSW RaBitQ can integrate cleanly without a broad HNSW refactor or a
premature cross-AM codec trait.

## Goal

Separate HNSW graph lifecycle logic from storage-format-specific payload,
metadata, and scoring logic. The immediate consumer is Task 63
(`storage_format = 'rabitq'` for `ec_hnsw`), but this task should preserve
existing TurboQuant and PqFastScan behavior exactly.

This implements the direction in ADR-072:

- shared quantizer families own quantization math;
- HNSW-local codec adapters own graph storage binding, tuple layout,
  reloption/metadata mapping, and traversal scoring hooks.

## Why

DiskANN Task 60 showed that a local codec-shaped adapter can keep a RaBitQ
integration small while preserving AM-specific invariants. HNSW is expected to
be more entangled than DiskANN because storage format touches:

- reloption parsing;
- graph storage descriptors;
- hot/cold/rerank tuple layout;
- build-time payload encoding;
- live insert payload encoding;
- vacuum scoring and payload retention;
- scan-time approximate scorer preparation;
- metadata compatibility for older TurboQuant and PqFastScan indexes.

Task 63 should not start by forcing HNSW through a cross-AM abstraction. It
should have a HNSW-local seam first.

## Scope

1. **Inventory current HNSW format coupling.** Map the TurboQuant and
   PqFastScan-specific code paths across build, insert, scan, vacuum, metadata,
   and graph storage descriptors.
2. **Define a HNSW-local codec adapter.** Start with concrete enums/helpers,
   not necessarily a trait. The adapter should expose only the hooks Task 63
   needs:
   - storage-format identity and reloption name;
   - metadata discriminator and compatibility validation;
   - graph storage descriptor selection;
   - build payload encoding;
   - insert payload encoding;
   - scan prepared-scorer construction;
   - insert/vacuum candidate scorer construction;
   - payload-retention information for vacuum.
3. **Move existing formats behind the adapter.** Route TurboQuant and
   PqFastScan through the new HNSW-local adapter without changing behavior.
4. **Preserve shared graph lifecycle.** Do not fork graph insertion, neighbor
   repair, or vacuum topology logic by format. Format-specific behavior should
   live at the payload/storage/scoring seam, consistent with ADR-033.
5. **Prepare Task 63 hooks.** Leave an obvious extension point for RaBitQ, but
   do not implement RaBitQ in this task unless Task 63 explicitly consumes the
   adapter on the same branch.
6. **Tests.** Add focused tests proving TurboQuant and PqFastScan still map to
   the same metadata, tuple layout, and scorer behavior as before. Prefer
   narrow PG18 checks when PostgreSQL behavior is touched.

## Non-Goals

- Do not add `storage_format = 'rabitq'`; Task 63 owns that.
- Do not extract a shared cross-AM codec trait; ADR-072 says to wait until
  DiskANN and HNSW both prove repeated shape.
- Do not change the default HNSW storage format.
- Do not optimize HNSW scan/build performance; Task 62 owns that lane.
- Do not change IVF or DiskANN adapters except for documentation references if
  needed.

## Acceptance Criteria

- HNSW TurboQuant and PqFastScan build, scan, insert, and vacuum behavior is
  unchanged after routing through the local codec adapter.
- The adapter boundary is documented in code or a packet so Task 63 can add
  RaBitQ without re-inventorying every HNSW storage-format branch.
- Existing HNSW metadata compatibility is preserved.
- Tests or review artifacts cover both existing formats on the touched paths.
- A review packet under `reviews/task-64/` records the adapter shape, touched
  files, validation, and explicit Task 63 handoff notes.

## Coordination

- **Task 63:** consumes this adapter for HNSW RaBitQ. Task 63 remains
  responsible for traversal viability, storage/recall benchmarks, and the
  actual RaBitQ storage format.
- **ADR-071:** defines the aspirational shared quantizer interface.
- **ADR-072:** defines why this task stays HNSW-local first.
- **ADR-033:** keeps graph lifecycle shared with format-specific adapters.
- **Task 42:** coordinate before any on-disk metadata or tuple compatibility
  change that affects format-invariant coverage.

## Stop Conditions

- Stop if the adapter would require changing on-disk bytes for existing
  TurboQuant or PqFastScan indexes.
- Stop if the extraction starts duplicating graph topology lifecycle logic
  instead of isolating payload/storage/scoring.
- Stop and update ADR-072 if HNSW proves a materially different shape that
  invalidates the codec-adapter boundary.
