# Task 94 Packet 008 Artifacts

- head SHA: `efeebf87c6a6a2e8c599da899235dab54c70c4c9`
- code checkpoint: `efeebf87c` (`Parse block kernel counters in bench suite results`)
- task bucket: `reviews/task-94/008-suite-block-kernel-counter-results/`
- lane: coder-1 LUT lane
- fixture: local `ecaz-cli` unit test
- storage format / quant: synthetic latency log with `pq_fastscan` / `grouped_pq`
- rerank mode: not applicable
- surface isolation: parser-only synthetic log fixture, no database table surface
- timestamp: `2026-06-09T10:44:05-07:00`

## Artifacts

### `suite-block-kernel-counter-parser-test.log`

- command: `cargo test -p ecaz-cli latency_result_rows_include_block_kernel_counter_lines`
- result: pass
- key result lines:
  - `running 1 test`
  - `test commands::bench::suite::tests::latency_result_rows_include_block_kernel_counter_lines ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 412 filtered out; finished in 0.00s`

## Evidence Notes

- This is local-only evidence. No CI and no AWS/Graviton 4 run was performed.
- The test fixture includes both a latency table row and a direct `[block-kernel-counters]` line.
- The parser now extracts the direct counter line as a `results.jsonl` row with metric `block_kernel_counters`, preserving `surface`, `quant`, `isa`, `kernel_candidates`, and `scalar_candidates`.
