# Task 148 Packet 005: IVF TQ+ Posting Calibration

## Summary

This checkpoint adds the IVF-local persistence and pure posting path for the Task 148 TurboQuant calibration profile.

Code commits:

- `026420a80` documents ADR-083, the IVF-specific calibration metadata layout.
- `365704672` wires `turboquant_profile = 'tqplus'` for pure `storage_format = 'turboquant'` postings.

Implemented:

- explicit `turboquant_profile = 'standard' | 'tqplus'` reloption, defaulting to `standard`;
- IVF metadata format bump with a TurboQuant calibration head and profile byte;
- dedicated `IvfTqCalibrationTuple` records for shift/scale arrays, not PQ codebook tuples;
- build-time calibration fit from `training_sample_rows`;
- deferred calibrated posting encoding after training;
- scan-time calibration model loading/caching and calibrated query prep/scoring;
- insert re-encoding from the persisted calibration model.

## Boundary

This checkpoint intentionally rejects `turboquant_profile = 'tqplus'` for `storage_format = 'coarse_rerank'` until the TurboQuant rerank sidecar codec is also made calibration-aware. That means the Task 148 stage2@25 measurement cell is not covered by this packet yet.

The calibrated pure-posting path currently scalar-falls back for scoring rather than using the existing no-QJL int8/LUT batch kernels. That is correct for behavior but is expected to be latency-negative until a calibrated batch epilogue lands.

## Validation

Packet-local log: `artifacts/cargo-check-ecaz-cli.log`

```text
cargo check -p ecaz-cli
```

Result: passed with existing warnings.

