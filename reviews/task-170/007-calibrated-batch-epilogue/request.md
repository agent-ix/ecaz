# Review Request: Task 170 Packet 007 - Calibrated TurboQuant Batch Epilogue

## Scope

This checkpoint removes the scalar-fallback limitation from packets 005/006.

- `PreparedTqCalibratedNoQjl4BitQuery` now stores the calibrated query LUT in the same i16 LUT form used by the existing no-QJL LUT32 batch scorer.
- Calibrated scoring reuses the existing LUT32 kernel and adds the calibration bias after each candidate score.
- IVF posting batches, contiguous sidecar slabs, and borrowed sidecar payload refs now dispatch calibrated TQ+ scoring through the batch wrapper.
- The temporary calibrated scalar loops in the sidecar rerank scorer were removed, so stage2 TQ+ sidecar batches exercise the batch dispatch.

This does not change payload bytes or metadata layout. It is intended to make Slice 3 measurement reflect the correction rather than a scalar fallback artifact.

## Validation

Artifacts are under `artifacts/`; see `artifacts/manifest.md`.

- `cargo check -p ecaz-cli` passed.
- `cargo test --release --lib turboquant_calibrated_sidecar_scores_scalar_and_batch_consistently` passed.
- `cargo test --release --lib calibration_no_qjl_4bit_reduces_anisotropic_score_error` passed.

## Remaining Task 170 Work

- Run Slice 3 A/B measurement with `ecaz bench suite` at 10k/50k/100k for both pure TQ default and stage2@25.
- Produce the keep/drop verdict for codebook calibration.
- No default flip and no push were performed.
