# Task 96 Packet 001 Artifact Manifest

- head SHA: `715b7fb2b` before this packet branch commit
- task bucket: `reviews/task-96/001-surface-inventory-stop-condition/`
- lane: coder-1 LUT lane, Task 96 TurboQuant no-QJL 2-bit
- fixture / storage format / rerank mode: none; static source inventory only
- host: local development checkout
- timestamp: 2026-06-09
- isolated one-index-per-table or shared-table surfaces: not applicable
- AWS / CI: not run

## Artifacts

### `surface-inventory.md`

- command: manually authored from the source audit logs below plus current task
  and ADR contracts
- key result: no AM exposes a real TurboQuant no-QJL 2-bit scoring surface;
  Task 96 stop condition is triggered

### `prod-quantizer-bits-audit.log`

- command: `script -q -c "rg -n 'DEFAULT_QUANT_BITS|validate_tqvector_bits|encode_to_ecvector expects|qjl_enabled|mse_bits|qjl_code_len_for_bits|prepare_ip_query_lut_no_qjl_4bit|MseNoQjl4Bit|PreparedLutNoQjl4BitQuery' src/lib.rs src/quant/prod.rs" reviews/task-96/001-surface-inventory-stop-condition/artifacts/prod-quantizer-bits-audit.log`
- key lines:
  - `src/lib.rs:348` defines `DEFAULT_QUANT_BITS: u8 = 4`
  - `src/lib.rs:367` validates `tqvector` bits against the canonical default
  - `src/quant/prod.rs:319` exposes only `prepare_ip_query_lut_no_qjl_4bit`
  - `src/quant/prod.rs:371` maps no-QJL exact mode only to `MseNoQjl4Bit`
  - `src/quant/prod.rs:1476` defines `qjl_enabled(...)`
  - `src/quant/prod.rs:1484` derives MSE bit count from QJL state

### `am-turboquant-surface-audit.log`

- command: `script -q -c "rg -n 'DEFAULT_QUANT_BITS|PreparedLutNoQjl4BitQuery|no_qjl_4bit|score_turboquant_no_qjl_4bit|ExactScoreMode::MseNoQjl4Bit|quant_bits == 4|bits: crate::DEFAULT_QUANT_BITS|bits == crate::DEFAULT_QUANT_BITS' src/am/ec_spire src/am/ec_ivf src/am/ec_diskann src/am/ec_hnsw" reviews/task-96/001-surface-inventory-stop-condition/artifacts/am-turboquant-surface-audit.log`
- key lines:
  - SPIRE stores `no_qjl_4bit_lut` and calls
    `score_turboquant_no_qjl_4bit_batch_for`
  - IVF exposes `TurboQuantNoQjl4BitLut` and gates scratch batching on
    `StorageFormat::TurboQuant => quant_bits == 4`
  - DiskANN uses `PreparedLutNoQjl4BitQuery` for TurboQuant prefilter scoring
  - HNSW exact modes and batch helpers are named no-QJL 4-bit

## Result

Task 96 should stop at Phase 0 until a real 2-bit no-QJL TurboQuant consumer is
introduced. No code, tests, benchmarks, AWS, or CI are needed for this stop
condition packet.
