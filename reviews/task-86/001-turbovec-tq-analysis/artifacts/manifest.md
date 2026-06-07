# Manifest: Task 86 Packet 001

- Head SHA: `71e16fcdced96714e7db1dd98f396cd68941180e`
- Follow-up addendum SHA: `d6462c594210e60e15fd9bb6b46f1f82508ee82f`
- Task bucket: `reviews/task-86/001-turbovec-tq-analysis/`
- Timestamp: `2026-06-07T05:41:49Z`
- Follow-up timestamp: `2026-06-07T19:25:00Z`
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
  - Follow-up addendum fixes stale `prod.rs` citation lines and adds the
    reviewer-requested 768/1536/3072 byte table, renormalization derivation,
    measurement methodology, and "not learnable from analysis alone" bounds.

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
