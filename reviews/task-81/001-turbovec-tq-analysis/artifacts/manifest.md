# Manifest: Task 81 Packet 001

- Head SHA: `39d72f3fdc5bd11114e8cc5269e7b62584f670a5`
- Task bucket: `reviews/task-81/001-turbovec-tq-analysis/`
- Timestamp: `2026-06-07T05:41:49Z`
- Lane: analysis only
- Fixture: none
- Storage format: TurboQuant only
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used:
  - `sed` / `rg` / `nl` source inspection against local TurboVec and current
    repo sources
- TurboVec source snapshot:
  - Local path: `/Users/peter/dev_bak/turbovec`
  - SHA: `efe29a184986cbf562a9847c2ac52a2990bfaca2`
  - Status: clean `main`

## Artifacts

- `turbovec-tq-analysis.md`
  - Analysis report comparing TurboVec TurboQuant implementation choices to our
    TurboQuant implementation only.

## Key Result Lines

- TurboVec is a flat compressed-vector scan index, not HNSW, DiskANN, IVF, or
  SPIRE.
- TurboVec query preparation rotates and inverse-calibrates the query, then
  builds LUTs. It does not pack the query into database code bytes.
- TurboVec per-vector bytes are approximately `dim * bits / 8 + 4`, plus
  index-level TQ+ calibration metadata.
- Our TurboQuant per-vector payload is `4 + mse_code_len(dim,bits) +
  qjl_code_len_for_bits(dim,bits)`, with QJL taking one bit per coordinate in
  lanes where it is enabled.
