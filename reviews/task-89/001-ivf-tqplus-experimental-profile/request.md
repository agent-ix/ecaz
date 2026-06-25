# Review Request: IVF TQ+ Experimental Calibration Profile

- Task: 89
- Packet: `reviews/task-89/001-ivf-tqplus-experimental-profile`
- Code commit: `0719aaaab01597a4b5ee1075823d44a592f45c83`
- Branch: `task-89-ivf-tqplus-profile`

## Summary

This checkpoint lands the Task 89 Phase 1 / partial Phase 2 shape for TQ+:

- Adds ADR-081, selecting an IVF-only experimental TurboQuant calibration profile instead of a public `turboquant_tqplus` storage format.
- Adds IVF reloption `turboquant_calibration = 'tqplus_experimental'`, valid only with `storage_format = 'turboquant'`.
- Persists the calibration profile in IVF metadata v10 while decoding v9 metadata as uncalibrated TurboQuant.
- Fits TQ+ calibration from deterministic IVF training vectors, persists shift/scale via the existing PQ-codebook tuple chain, and reloads it for insert/scan.
- Encodes IVF postings in calibrated rotated TurboQuant space and prepares/scans with inverse calibration and bias handling.
- Adds focused deterministic calibration, scorer formula, option parsing, metadata compatibility, and size invariant coverage.

## What This Does Not Claim

This is not Task 89 completion. It does not include:

- DBPedia 10k/50k/100k A/B benchmark evidence.
- QJL/gamma-aware TQ+ mode implementation or measurement.
- Streaming-insert drift measurements.
- Cross-corpus measurements.
- Public shape gate decision.
- SPIRE/HNSW/DiskANN ports.

The implementation currently supports the no-QJL 4-bit experimental path. The QJL/gamma-aware path needs a separate design pass because TQ+ scoring needs both residual gamma and candidate renorm scalar; the existing TurboQuant posting scalar can only carry one value without changing payload shape.

## Validation

Artifacts are recorded in `artifacts/manifest.md`.

- `cargo check -p ecaz --lib --no-default-features --features pg18` passed.
- `cargo test -p ecaz --lib --no-default-features --features pg18 tqplus_` passed.
- `cargo test -p ecaz --lib --no-default-features --features pg18 metadata_roundtrip` passed.
- `cargo test -p ecaz --test size_of_assertions --no-default-features --features pg18` passed.

## Review Focus

- ADR-081 shape: is `turboquant_calibration = 'tqplus_experimental'` the right internal/experimental knob for Phase 1?
- IVF metadata compatibility: v10 writes byte 92; v9 decodes as `TurboQuantCalibration::None`.
- Calibration persistence: shift/scale reuse the existing `IvfPqCodebookTuple` chain under `pq_codebook_head`.
- Scoring correctness for the no-QJL 4-bit calibrated path.
- Whether QJL/gamma-aware TQ+ should use an appended per-vector renorm scalar, a packed scalar pair, or a separate experimental payload shape.
