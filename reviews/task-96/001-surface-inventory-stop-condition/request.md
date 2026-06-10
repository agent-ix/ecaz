# Task 96 Packet 001: Surface Inventory Stop Condition

Task 96 begins with a mandatory Phase 0 surface inventory. The task file says
to file a Stop Condition packet immediately and avoid speculative kernels if no
AM exposes a real TurboQuant no-QJL 2-bit scoring surface.

## Verdict

Stop condition triggered.

No current AM exposes TurboQuant no-QJL 2-bit scoring. The only no-QJL LUT
surface in tree is the canonical 4-bit lane from Task 87:
`PreparedLutNoQjl4BitQuery` plus `score_turboquant_no_qjl_4bit_batch_for(...)`.

## Evidence

Packet-local artifacts:

- `artifacts/surface-inventory.md`
- `artifacts/prod-quantizer-bits-audit.log`
- `artifacts/am-turboquant-surface-audit.log`
- `artifacts/manifest.md`

Key source facts:

- `ProdQuantizer::qjl_enabled(dim, bits)` disables QJL only for `bits == 4`
  with a supported tiled dimension. Therefore `bits == 2` is QJL-enabled today.
- `mse_bits(dim, bits)` uses `bits - 1` when QJL is enabled, so the current
  2-bit lane is a 1-bit MSE + QJL residual-sign lane, not no-QJL 2-bit.
- The explicit no-QJL LUT APIs are all 4-bit:
  `prepare_ip_query_lut_no_qjl_4bit`,
  `score_ip_from_parts_lut_no_qjl_4bit`, and
  `mse_code_bytes_no_qjl_4bit`.
- `tqvector` and `ecvector` quantizer entry points enforce canonical
  `DEFAULT_QUANT_BITS = 4`, so the SQL-visible TurboQuant surface does not
  publish a 2-bit no-QJL index/scoring path.
- SPIRE, IVF, DiskANN, and HNSW all route no-QJL batch scoring through
  `PreparedLutNoQjl4BitQuery` and `score_turboquant_no_qjl_4bit_batch_for(...)`.

## Review Ask

Please review this as the Task 96 Phase 0 stop-condition packet.

If accepted, Task 96 should be considered deferred until a separate storage /
scoring-surface task introduces a real no-QJL 2-bit TurboQuant consumer. The
current 2-bit TurboQuant path is QJL-enabled and belongs to Task 97's
gamma/residual-sign kernel family instead.

No tests, benchmarks, AWS, or CI were run because the task file explicitly
requires stopping before speculative implementation when no consumer exists.
