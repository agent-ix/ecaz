# Task 86 Packet 004 Artifact Manifest

- Head SHA: `462ece1df410ff2e3389c85c213e185321ae9d07`
- Task bucket: `reviews/task-86/004-byte-lut-kernel`
- Timestamp: `2026-06-07T06:50:24Z`
- Lane: focused unit/prototype kernel probe, not an accepted benchmark lane
- Fixture: deterministic synthetic 1536-dimensional inner-product corpus, 512 candidates, 32 repeated scans
- Storage format: in-memory TQ no-QJL 4-bit packed MSE bytes
- Rerank mode: none
- Index surface: quantizer-only scorer probe; no HNSW, DiskANN, IVF, or SPIRE index build
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `byte-lut-probe.log`

Command:

```sh
cargo test -p ecaz --lib --no-default-features --features pg18 quant::prod::tests::byte_lut_no_qjl_4bit_probe_reports_kernel_delta -- --nocapture > reviews/task-86/004-byte-lut-kernel/artifacts/byte-lut-probe.log 2>&1
```

Key result:

```text
task86_byte_lut_probe dim=1536 candidates=512 repeats=32 scores=16384 direct_ns_per_score=9356.24 dim_lut_ns_per_score=4448.95 byte_lut_ns_per_score=5458.79 byte_lut_speedup_vs_direct=1.714 byte_lut_speedup_vs_dim_lut=0.815 checksum=65.023254
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1977 filtered out; finished in 0.70s
```
