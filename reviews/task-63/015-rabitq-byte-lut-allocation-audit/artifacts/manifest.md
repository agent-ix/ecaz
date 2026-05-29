# Artifact Manifest: Task 63 RaBitQ Byte-LUT Allocation Audit

- head SHA: `554892259999e5f4417fcd8198cf8c55ee81c226`
- task bucket: `reviews/task-63/`
- packet path: `reviews/task-63/015-rabitq-byte-lut-allocation-audit/`
- lane: common RaBitQ scorer allocation audit
- timestamp: 2026-05-27 America/Los_Angeles

## Artifacts

| Artifact | Command | Result |
| --- | --- | --- |
| `cargo-test-prepared-query-byte-lut-no-run.log` | `cargo test -q --lib prepared_queries_only_keep_bits1_byte_lut_for_bits1 --no-run` | passed compile/no-run |
| `cargo-test-prepared-query-byte-lut-runtime.log` | `cargo test -q --lib prepared_queries_only_keep_bits1_byte_lut_for_bits1` | blocked locally by `undefined symbol: LockBuffer` |

## Notes

This packet is audit-only. It confirms current code avoids retaining a bits=1
byte LUT for non-1-bit prepared queries and does not run any benchmark matrix.
