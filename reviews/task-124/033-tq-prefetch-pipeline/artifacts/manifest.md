# Task 124 / 033 TQ Payload Prefetch Manifest

- head SHA: `cfb209bba8454461fe3a6abed5fd71502de58263`
- task bucket: `reviews/task-124/033-tq-prefetch-pipeline`
- lane: local release unit/profiler
- fixture: synthetic 1536-dim TurboQuant no-QJL 4-bit LUT32 scorer inputs
- storage format: TurboQuant no-QJL 4-bit scorer payload code
- rerank mode: scorer-only no-QJL LUT32 batch path
- timestamp: 2026-06-30
- isolated one-index-per-table: not applicable; scorer-only microprofile

## Artifacts

### `tq-prefetch-profile.log`

Command:

```sh
ECAZ_TQ_BATCH_WIDTH_PROFILE_LOG=reviews/task-124/033-tq-prefetch-pipeline/artifacts/tq-prefetch-profile.log cargo test --release --lib --features bench task124_profile_tq_no_qjl_flush_widths -- --ignored --nocapture
```

Key result lines:

```text
task124_tq_batch_width_profile backend=neon dim=1536 total_candidates=256000
width=8 ns_per_candidate=253.4 prefetch_ns_per_candidate=235.1
width=16 ns_per_candidate=232.5 prefetch_ns_per_candidate=232.7
width=25 ns_per_candidate=295.2 prefetch_ns_per_candidate=295.5
width=32 ns_per_candidate=231.7 prefetch_ns_per_candidate=231.3
width=64 ns_per_candidate=232.3 prefetch_ns_per_candidate=231.9
width=96 ns_per_candidate=232.1 prefetch_ns_per_candidate=232.3
width=100 ns_per_candidate=244.1 prefetch_ns_per_candidate=241.3
width=128 ns_per_candidate=232.8 prefetch_ns_per_candidate=233.6
```

Result: mixed/noisy, not enabled in production. The production no-QJL scorer
continues to call the original unprefetched width cascade.

### `validation.log`

Commands:

```sh
cargo fmt --check
cargo test --release --lib --features bench turboquant_lut_batch_matches_scalar_tail -- --nocapture
```

Key result lines:

```text
cargo fmt --check: passed (stable rustfmt warns that repo rustfmt.toml has nightly-only import options)
turboquant_lut_batch_matches_scalar_tail: 1 passed; 0 failed
```
