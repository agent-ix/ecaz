# Task 89 / Packet 002: Cross-AM TQ+ Implementation Inventory

## Summary

This packet records the current all-AM implementation inventory after the
Phase 1 ADR packet. No porting code is included.

Key finding: DiskANN currently does **not** support baseline TurboQuant. It has
only `pq_fastscan` and `rabitq` storage formats. Because Task 89 requires IVF,
SPIRE, HNSW, and DiskANN to compare TQ+ against TQ, the DiskANN scope includes
adding a baseline TurboQuant DiskANN codec before adding TQ+.

## Artifact

- `artifacts/cross-am-tqplus-inventory.md`

## Validation

Documentation-only inventory. No tests were run.

Source inspection covered:

- `src/am/ec_ivf/options.rs`
- `src/am/ec_ivf/quantizer.rs`
- `src/am/ec_spire/options/mod.rs`
- `src/am/ec_hnsw/options.rs`
- `src/am/ec_hnsw/graph.rs`
- `src/am/ec_diskann/options.rs`
- `src/am/ec_diskann/page.rs`
- `src/am/ec_diskann/quantizer.rs`
- `src/quant/prod.rs`
- `src/quant/mod.rs`

## Reviewer Focus

Please confirm whether the identified DiskANN baseline-TurboQuant gap should
be treated as part of Task 89 implementation scope, as the task's all-index
validation requirement implies.
