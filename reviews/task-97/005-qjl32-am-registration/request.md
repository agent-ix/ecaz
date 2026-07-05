# Task 97 Packet 005: qjl32 AM Registration

This packet registers the existing qjl32 batch helper against the current
production QJL/gamma-aware AM paths for IVF, SPIRE, and HNSW. It follows the
reviewer-clarified Task 97 scope: canonical `bits=4` TurboQuant QJL at
non-tiled dimensions such as 1024, no new TQ mode, no new storage surface, and
DiskANN remains out of scope because its current TurboQuant path is no-QJL only.

## Changes

- IVF:
  - Renames the raw TurboQuant payload batch helper to
    `score_turboquant_batch_from_payloads`.
  - Routes `IvfPreparedQuery::TurboQuant` through
    `score_turboquant_qjl_batch_for`.
  - Keeps `TurboQuantNoQjl4BitLut` on the existing lut32 no-QJL helper.
- SPIRE:
  - Routes both raw assignment batch scoring and `CandidateBatch` scoring to
    qjl32 when `no_qjl_4bit_lut.is_none()`.
  - Records scalar fallback rows as `QuantCodecKind::TurboQuantQjl` for QJL
    scalar paths.
- HNSW:
  - Adds `HnswTurboQuantScanCodec::score_ip_batch`.
  - Routes exact QJL-active prepared queries to qjl32 and full-LUT no-QJL
    prepared queries to the existing lut32 helper.
- Shared batch helper:
  - Accepts both `CandidateMeta::Gamma` and
    `CandidateMeta::GammaAndResidualSigns { gamma, .. }` for qjl32 while
    continuing to read residual signs from packed payload bytes.

## Validation

- `cargo test qjl32 --lib -- --color never`
  - 9 passed; 0 failed
- `cargo test turboquant_qjl --lib -- --color never`
  - 6 passed; 0 failed
- `cargo test common_quant_codec_turboquant_batch_is_bit_exact_with_scalar --lib -- --color never`
  - 1 passed; 0 failed

Logs are under `artifacts/`.

## Review Request

Please review the IVF/SPIRE/HNSW qjl32 registration paths, the
`GammaAndResidualSigns` metadata acceptance in the shared helper, and the
counter attribution tests for `quant=turboquant_qjl`. No CI or AWS validation
was run for this packet.
