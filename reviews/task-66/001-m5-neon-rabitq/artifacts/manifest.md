# Task 66 Packet 001 Artifact Manifest

- head SHA: `91f2e51ec9e7825f4f6710eac11bbe75408281fe`
- task bucket: `reviews/task-66/001-m5-neon-rabitq`
- timestamp: `2026-05-29T08:19:22-0700`
- host lane: Apple Silicon M5 laptop, local Criterion microbench
- fixture/storage/rerank: in-process RaBitQ quantizer scoring, dim=1536, bits={1,4,8}, no PostgreSQL table surface
- isolated one-index-per-table: not applicable; pure quantizer benchmark

## Artifacts

### `criterion-rabitq-neon.log`

- command:
  `cargo bench --features bench --bench quant_score -- quant/rabitq_score --sample-size 10 --measurement-time 1`
- key result lines:
  - bits1 single: `time: [87.586 ns 87.855 ns 88.241 ns]`
  - bits1 batch1000: `time: [85.175 us 85.877 us 86.371 us]`
  - bits4 single: `time: [233.24 ns 235.99 ns 240.52 ns]`
  - bits8 single: `time: [123.71 ns 124.95 ns 127.05 ns]`
  - bits8 batch1000: `time: [123.03 us 124.17 us 125.53 us]`
  - bits8c3 single: `time: [123.07 ns 124.14 ns 125.31 ns]`
  - bits8c3 batch1000: `time: [122.33 us 123.24 us 124.43 us]`
  - bits8c4 single: `time: [121.25 ns 121.77 ns 122.37 ns]`
  - bits8c4 batch1000: `time: [120.78 us 122.47 us 125.11 us]`

### `criterion-rabitq-bf16.log`

- command:
  `cargo bench --features 'bench rabitq-bf16' --bench quant_score -- quant/rabitq_score/bits4 --sample-size 10 --measurement-time 1`
- key result lines:
  - bf16-enabled bits4: `time: [233.02 ns 235.20 ns 239.31 ns]`
  - Criterion comparison: `No change in performance detected.`

## Validation Commands

- `cargo check --no-default-features --features pg18`
- `cargo test --lib --no-default-features --features pg18 quant::rabitq`
- `cargo check -p ecaz-cli`
- `cargo check --benches --features bench --no-default-features --features pg18`
