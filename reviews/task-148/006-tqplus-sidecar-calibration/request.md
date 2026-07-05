# Review Request: Task 148 Packet 006 - TQ+ Calibration Sidecar Rerank

## Scope

This checkpoint wires the Task 148 codebook-calibration profile through the stage2 TurboQuant sidecar cell:

- `turboquant_profile = 'tqplus'` is now accepted for `storage_format = 'coarse_rerank'`, `rerank_placement = 'index'`, `rerank_format = 'turboquant'`.
- Build-time TQ+ calibration training now runs after IVF assignment so pure TQ postings fit on source vectors, while stage2 TurboQuant sidecars fit on residual vectors (`source - assigned_centroid`).
- The compact rerank sidecar encoder/scorer carries the persisted TQ calibration model for TurboQuant payload encode and query prep.
- Insert-side sidecar appends load the persisted model before encoding new TQ+ sidecar payloads.
- Scan-side stage2 rerank loads the model and resolves a calibrated TurboQuant scorer.
- Calibrated stage2 sidecar batch scoring currently scalar-falls back inside the rerank batch wrappers for correctness; latency-neutral batch epilogue work is still pending before measurement.

No on-disk payload-width change is introduced by this checkpoint. It uses the ADR-083 calibration metadata persisted in packet 005.

## Validation

Artifacts are under `artifacts/`; see `artifacts/manifest.md`.

- `cargo check -p ecaz-cli` passed.
- `cargo test --release --lib coarse_rerank_accepts_tqplus_turboquant_sidecar_profile` passed.
- `cargo test --release --lib turboquant_calibrated_sidecar_scores_scalar_and_batch_consistently` passed.

## Remaining Task 148 Work

- Run Slice 3 A/B measurement with `ecaz bench suite` at 10k/50k/100k for both pure TQ default and stage2@25.
- Decide whether to add a calibrated batch epilogue before measurement; the current scalar fallback is likely latency-negative for the stage2 TQ+ sidecar gate.
- No default flip and no push were performed.
