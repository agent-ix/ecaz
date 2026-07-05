# Task 97 Packet 015 Artifact Manifest

- Head SHA: `7c4935018dd3564f3e4b9ffe13bbc9ef2980df3f`
- Task bucket: `reviews/task-97/015-old-scorer-diagnostic/`
- Lane: coder-1 / Task 97 TurboQuant QJL block kernel
- Fixture: local x86_64 AVX2/FMA host, `ProdQuantizer::new(1024, 4, 42)`
- Storage format / rerank mode: diagnostic per-candidate QJL scorer only;
  no AM index storage, no AWS, no GitHub CI
- Timestamp: `2026-06-10T04:32:37Z`

## Artifacts

### `local-cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Result: passed
- Note: stable rustfmt emitted the repository's usual unstable-option warnings.

### `local-git-diff-check.log`

- Command: `git diff --check`
- Result: passed

### `local-cargo-test-old-qjl-tolerance.log`

- Command: `cargo test qjl_pre_b0efa19d9_multi_accum_tolerance_diagnostic --lib -- --nocapture --color never`
- Result: passed
- Key line:
  - `qjl_pre_b0efa19d9_multi_accum_tolerance_diagnostic dim=1024 bits=4 candidates=1000 max_ulp=5920 max_rel=4.106507404e-4 violations=285 worst_seed=924`

### `local-cargo-test-qjl-meta-existing.log`

- Command: `cargo test turboquant_qjl_batch_matches_pre_slice_scalar_reference_and_records_counters --lib -- --nocapture --color never`
- Result: passed
- Purpose: covers the existing QJL batch metadata path after the packet 005
  `GammaAndResidualSigns` clarification comment.

### `local-cargo-bench-score-ip-from-parts-d1024-old-vs-new.log`

- Command: `cargo bench --features bench --bench quant_score 'd1024_b4' -- --sample-size 10 --warm-up-time 1 --measurement-time 2`
- Result: passed
- Key rows:
  - current `quant/score_ip_from_parts/d1024_b4/1024`: `[872.98 ns 877.79 ns 884.19 ns]`
  - old diagnostic `quant/score_ip_from_parts/pre_b0efa19d9_multi_accum/d1024_b4`: `[257.62 ns 267.76 ns 276.49 ns]`
  - qjl32 scalar block row: `[34.906 us 36.034 us 37.182 us]`
  - qjl32 dispatch block row: `[28.187 us 28.587 us 29.065 us]`

## Summary

The old pre-`b0efa19d9` multi-accumulator per-candidate scorer was faster
but not tolerance-clean at the Task 97 production QJL fixture. The current
production scorer remains the correctness anchor; packet 011's required
qjl32 AVX2 candidate-parallel transpose is a separate block-kernel slice.
